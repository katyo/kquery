use crate::{FileMgr, ModuleData, Path, Result, SourceData};

use clex::{Lexer, Token};
use std::collections::BTreeSet as Set;
use tokio::io::AsyncReadExt;

impl SourceData {
    pub async fn from_source(filemgr: &FileMgr, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = filemgr.open(&path).await?;
        let mut src = String::new();

        file.read_to_string(&mut src).await?;

        let lexer = Lexer::from(src.as_ref());

        let mut compat_strs = Set::default();
        let mut module = ModuleData::default();

        #[derive(Clone)]
        enum State {
            TopLevel,

            // compat string
            Dot,
            DotCompat,
            DotCompatEq,
            DotCompatEqString {
                string: String,
            },

            // module string
            ModuleStr {
                name: ModuleStr,
            },
            ModuleStrLParen {
                name: ModuleStr,
            },
            ModuleStrLParenStr {
                name: ModuleStr,
                string: String,
            },

            // module param
            ModulePar {
                usafe: bool,
                named: bool,
            },
            ModuleParLParen {
                usafe: bool,
                named: bool,
            },
            ModuleParLParenName {
                usafe: bool,
                named: bool,
                name: String,
            },
            ModuleParLParenNameComma {
                usafe: bool,
                named: bool,
                name: String,
            },
            // named only
            ModuleParLParenNameCommaVar {
                usafe: bool,
                named: bool,
                name: String,
            },
            // named only
            ModuleParLParenNameCommaVarComma {
                usafe: bool,
                named: bool,
                name: String,
            },
            ModuleParLParenNameCommaType {
                usafe: bool,
                named: bool,
                name: String,
                type_: String,
            },
            ModuleParLParenNameCommaTypeComma {
                usafe: bool,
                named: bool,
                name: String,
                type_: String,
            },
            ModuleParLParenNameCommaTypeCommaPerm {
                usafe: bool,
                named: bool,
                name: String,
                type_: String,
                perm: u16,
            },

            // module param description
            ModuleParDesc,
            ModuleParDescLParen,
            ModuleParDescLParenName {
                name: String,
            },
            ModuleParDescLParenNameComma {
                name: String,
            },
            ModuleParDescLParenNameCommaStr {
                name: String,
                string: String,
            },
        }

        #[derive(Clone, Copy)]
        #[repr(u8)]
        enum ModuleStr {
            Description,
            Author,
            License,
            Alias,
        }

        let mut state = State::TopLevel;

        for lexeme in lexer.filter(|lexeme| lexeme.token != Token::Comment) {
            match state {
                State::TopLevel => match lexeme.token {
                    // find compat strings
                    Token::Symbol if lexeme.slice == "." => {
                        state = State::Dot;
                        continue;
                    }
                    Token::Identifier => {
                        if lexeme.slice.starts_with("MODULE_") {
                            match &lexeme.slice[7..] {
                                "DESCRIPTION" => {
                                    state = State::ModuleStr {
                                        name: ModuleStr::Description,
                                    };
                                    continue;
                                }
                                "LICENSE" => {
                                    state = State::ModuleStr {
                                        name: ModuleStr::License,
                                    };
                                    continue;
                                }
                                "AUTHOR" => {
                                    state = State::ModuleStr {
                                        name: ModuleStr::Author,
                                    };
                                    continue;
                                }
                                "ALIAS" => {
                                    state = State::ModuleStr {
                                        name: ModuleStr::Alias,
                                    };
                                    continue;
                                }
                                "PARM_DESC" => {
                                    state = State::ModuleParDesc;
                                    continue;
                                }
                                _ => {}
                            }
                        } else if lexeme.slice.starts_with("module_param") {
                            match &lexeme.slice[12..] {
                                "" => {
                                    state = State::ModulePar {
                                        usafe: false,
                                        named: false,
                                    };
                                    continue;
                                }
                                "_unsafe" => {
                                    state = State::ModulePar {
                                        usafe: true,
                                        named: false,
                                    };
                                    continue;
                                }
                                "_named" => {
                                    state = State::ModulePar {
                                        usafe: false,
                                        named: true,
                                    };
                                    continue;
                                }
                                "_named_unsafe" => {
                                    state = State::ModulePar {
                                        usafe: true,
                                        named: true,
                                    };
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },

                // compat string
                State::Dot => {
                    // .compatible
                    if lexeme.token == Token::Identifier && lexeme.slice == "compatible" {
                        state = State::DotCompat;
                        continue;
                    }
                }
                State::DotCompat => {
                    // .compatible =
                    if lexeme.token == Token::Symbol && lexeme.slice == "=" {
                        state = State::DotCompatEq;
                        continue;
                    }
                }
                State::DotCompatEq => {
                    if lexeme.token == Token::String {
                        if let Some(string) = lexeme.string() {
                            state = State::DotCompatEqString { string };
                            continue;
                        }
                    }
                }
                State::DotCompatEqString { string } => {
                    if lexeme.token == Token::Symbol && (lexeme.slice == "," || lexeme.slice == "}")
                    {
                        compat_strs.insert(string);
                    }
                }

                // module string
                State::ModuleStr { name } => {
                    // MODULE_<name>(
                    if lexeme.token == Token::Symbol && lexeme.slice == "(" {
                        state = State::ModuleStrLParen { name };
                        continue;
                    }
                }
                State::ModuleStrLParen { name } => {
                    // MODULE_<name>("str"
                    if lexeme.token == Token::String {
                        if let Some(string) = lexeme.string() {
                            state = State::ModuleStrLParenStr { name, string };
                            continue;
                        }
                    }
                }
                State::ModuleStrLParenStr { name, string } => {
                    // MODULE_<name>("str")
                    if lexeme.token == Token::Symbol && lexeme.slice == ")" {
                        match name {
                            ModuleStr::Description => module.description = string,
                            ModuleStr::Author => module.authors.push(string),
                            ModuleStr::License => module.license = string,
                            ModuleStr::Alias => module.aliases.push(string),
                        }
                    }
                }

                // module param
                State::ModuleParDesc => {
                    if lexeme.token == Token::Symbol && lexeme.slice == "(" {
                        state = State::ModuleParDescLParen;
                        continue;
                    }
                }
                State::ModuleParDescLParen => {
                    if lexeme.token == Token::Identifier {
                        state = State::ModuleParDescLParenName {
                            name: lexeme.slice.into(),
                        };
                        continue;
                    }
                }
                State::ModuleParDescLParenName { name } => {
                    if lexeme.token == Token::Symbol && lexeme.slice == "," {
                        state = State::ModuleParDescLParenNameComma { name };
                        continue;
                    }
                }
                State::ModuleParDescLParenNameComma { name } => {
                    if lexeme.token == Token::String {
                        if let Some(string) = lexeme.string() {
                            state = State::ModuleParDescLParenNameCommaStr { name, string };
                            continue;
                        }
                    }
                }
                State::ModuleParDescLParenNameCommaStr { name, string } => {
                    if lexeme.token == Token::Symbol && lexeme.slice == ")" {
                        module
                            .params
                            .entry(name)
                            .or_insert_with(Default::default)
                            .description = string;
                    }
                }

                // module param
                State::ModulePar { usafe, named } => {
                    if lexeme.token == Token::Symbol && lexeme.slice == "(" {
                        state = State::ModuleParLParen { usafe, named };
                        continue;
                    }
                }
                State::ModuleParLParen { usafe, named } => {
                    if lexeme.token == Token::Identifier {
                        state = State::ModuleParLParenName {
                            usafe,
                            named,
                            name: lexeme.slice.into(),
                        };
                        continue;
                    }
                }
                State::ModuleParLParenName { usafe, named, name } => {
                    if lexeme.token == Token::Symbol && lexeme.slice == "," {
                        state = State::ModuleParLParenNameComma { usafe, named, name };
                        continue;
                    }
                }
                State::ModuleParLParenNameComma { usafe, named, name } => {
                    if lexeme.token == Token::Symbol {
                        if named {
                            state = State::ModuleParLParenNameCommaVar { usafe, named, name };
                        } else {
                            state = State::ModuleParLParenNameCommaType {
                                usafe,
                                named,
                                name,
                                type_: lexeme.slice.into(),
                            };
                        }
                        continue;
                    }
                }
                State::ModuleParLParenNameCommaVar { usafe, named, name } => {
                    if lexeme.token == Token::Symbol && lexeme.slice == "," {
                        state = State::ModuleParLParenNameCommaVarComma { usafe, named, name };
                        continue;
                    }
                }
                State::ModuleParLParenNameCommaVarComma { usafe, named, name } => {
                    if lexeme.token == Token::Identifier {
                        state = State::ModuleParLParenNameCommaType {
                            usafe,
                            named,
                            name,
                            type_: lexeme.slice.into(),
                        };
                        continue;
                    }
                }
                State::ModuleParLParenNameCommaType {
                    usafe,
                    named,
                    name,
                    type_,
                } => {
                    if lexeme.token == Token::Symbol && lexeme.slice == "," {
                        state = State::ModuleParLParenNameCommaTypeComma {
                            usafe,
                            named,
                            name,
                            type_,
                        };
                        continue;
                    } else if lexeme.token == Token::Identifier {
                        let mut type_ = type_;
                        type_.push(' ');
                        type_.push_str(lexeme.slice);
                        state = State::ModuleParLParenNameCommaType {
                            usafe,
                            named,
                            name,
                            type_,
                        };
                        continue;
                    }
                }
                State::ModuleParLParenNameCommaTypeComma {
                    usafe,
                    named,
                    name,
                    type_,
                } => {
                    if lexeme.token == Token::Identifier {
                        state = State::ModuleParLParenNameCommaTypeCommaPerm {
                            usafe,
                            named,
                            name,
                            type_,
                            perm: mode_from_id(lexeme.slice),
                        };
                        continue;
                    } else if lexeme.token == Token::Int {
                        if let Some(perm) = lexeme.int() {
                            state = State::ModuleParLParenNameCommaTypeCommaPerm {
                                usafe,
                                named,
                                name,
                                type_,
                                perm,
                            };
                            continue;
                        }
                    }
                }
                State::ModuleParLParenNameCommaTypeCommaPerm {
                    usafe,
                    named,
                    name,
                    type_,
                    perm,
                } => {
                    if lexeme.token == Token::Symbol {
                        if lexeme.slice == "|" {
                            state = State::ModuleParLParenNameCommaTypeCommaPerm {
                                usafe,
                                named,
                                name,
                                type_,
                                perm,
                            };
                            continue;
                        } else if lexeme.slice == ")" {
                            let mut par =
                                module.params.entry(name).or_insert_with(Default::default);
                            par.type_ = type_;
                            par.perm = perm;
                        }
                    } else if lexeme.token == Token::Identifier {
                        state = State::ModuleParLParenNameCommaTypeCommaPerm {
                            usafe,
                            named,
                            name,
                            type_,
                            perm: perm | mode_from_id(lexeme.slice),
                        };
                        continue;
                    } else if lexeme.token == Token::Int {
                        if let Some(extra_perm) = lexeme.int::<u16>() {
                            state = State::ModuleParLParenNameCommaTypeCommaPerm {
                                usafe,
                                named,
                                name,
                                type_,
                                perm: perm | extra_perm,
                            };
                            continue;
                        }
                    }
                }
            }
            state = State::TopLevel;
        }

        Ok(Self {
            compat_strs,
            module: if module.is_empty() {
                None
            } else {
                Some(module)
            },
            ..Default::default()
        })
    }
}

fn mode_from_id(id: &str) -> u16 {
    if let Some(sfx) = id.strip_prefix("S_I") {
        match sfx {
            "FMT" => 0o0170000,
            "FSOCK" => 0o140000,
            "FLNK" => 0o120000,
            "FREG" => 0o100000,
            "FBLK" => 0o060000,
            "FDIR" => 0o040000,
            "FCHR" => 0o020000,
            "FIFO" => 0o010000,
            "SUID" => 0o004000,
            "SGID" => 0o002000,
            "SVTX" => 0o001000,

            "RWXU" => 0o0700,
            "RUSR" => 0o0400,
            "WUSR" => 0o0200,
            "XUSR" => 0o0100,

            "RWXG" => 0o0070,
            "RGRP" => 0o0040,
            "WGRP" => 0o0020,
            "XGRP" => 0o0010,

            "RWXO" => 0o0007,
            "ROTH" => 0o0004,
            "WOTH" => 0o0002,
            "XOTH" => 0o0001,

            "RWXUGO" => 0o0777,
            "ALLUGO" => 0o007777,
            "RUGO" => 0o0444,
            "WUGO" => 0o0222,
            "XUGO" => 0o0111,

            _ => 0,
        }
    } else {
        0
    }
}
