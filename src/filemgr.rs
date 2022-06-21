use crate::{Path, PathBuf, Result};

use std::{
    io,
    io::SeekFrom,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncSeek, AsyncWrite, ReadBuf},
    sync::{OwnedSemaphorePermit, Semaphore},
};

/// File instance
#[derive(Debug, educe::Educe)]
#[educe(Deref, DerefMut)]
#[pin_project::pin_project]
pub struct File {
    perm: OwnedSemaphorePermit,
    #[educe(Deref, DerefMut)]
    #[pin]
    file: tokio::fs::File,
}

impl AsyncRead for File {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        dst: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().file.poll_read(cx, dst)
    }
}

impl AsyncSeek for File {
    fn start_seek(self: Pin<&mut Self>, pos: SeekFrom) -> io::Result<()> {
        self.project().file.start_seek(pos)
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        self.project().file.poll_complete(cx)
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        src: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().file.poll_write(cx, src)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().file.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().file.poll_flush(cx)
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for File {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.file.as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for File {
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.file.as_raw_handle()
    }
}

/// File manager
#[derive(Debug, Clone)]
pub struct FileMgr {
    /// Base directory
    dir: Arc<PathBuf>,

    /// Open files semaphore
    sem: Arc<Semaphore>,
}

impl FileMgr {
    /// Create file manager instance using specified base directory
    pub fn new(dir: impl Into<PathBuf>) -> Result<Self> {
        let (soft, hard) = rlimit::Resource::NOFILE.get()?;
        let max_open_files = soft.min(hard) - 10;

        log::debug!("Max open files: {}", max_open_files);

        Ok(Self {
            dir: Arc::new(dir.into()),
            sem: Arc::new(Semaphore::new(max_open_files as _)),
        })
    }

    /// Get base directory path
    pub fn base_path(&self) -> &PathBuf {
        &self.dir
    }

    /// Get full path to files in base directory using relative path
    pub fn full_path(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = path.as_ref();
        if path.is_absolute() {
            anyhow::bail!("Path should be relative: {}", path.display());
        }
        Ok(self.dir.join(path))
    }

    /// Check directory existing in base directory using relative path
    pub async fn dir_exists(&self, path: impl AsRef<Path>) -> Result<bool> {
        Ok(tokio::fs::metadata(self.full_path(path)?)
            .await
            .map(|m| m.is_dir())
            .unwrap_or(false))
    }

    /// Check file existing in base directory using relative path
    pub async fn file_exists(&self, path: impl AsRef<Path>) -> Result<bool> {
        Ok(tokio::fs::metadata(self.full_path(path)?)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false))
    }

    /// Open file in base directory using relative path
    pub async fn open(&self, path: impl AsRef<Path>) -> Result<File> {
        let perm = Semaphore::acquire_owned(self.sem.clone()).await?;
        let file = tokio::fs::File::open(self.full_path(path)?).await?;

        Ok(File { perm, file })
    }

    /// Create file in base directory using relative path
    pub async fn create(&self, path: impl AsRef<Path>) -> Result<File> {
        let perm = Semaphore::acquire_owned(self.sem.clone()).await?;
        let file = tokio::fs::File::create(self.full_path(path)?).await?;

        Ok(File { perm, file })
    }
}
