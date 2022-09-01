use std::{future::Future, io::Result};
use tokio::{fs::File, io::AsyncWriteExt};

pub struct LaTeXWriter {
    file: File,
}

impl LaTeXWriter {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    pub async fn write(&mut self, text: impl AsRef<str>) -> Result<&mut Self> {
        self.file.write_all(text.as_ref().as_bytes()).await?;
        Ok(self)
    }

    pub async fn command(
        &mut self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<&mut Self> {
        self.file
            .write_all(
                format!("\\{}{}\n", cmd.as_ref(), unsafe {
                    String::from_utf8_unchecked(
                        args.into_iter()
                            .map(|arg| format!("{{{}}}", arg.as_ref()))
                            .flat_map(|s| s.into_bytes())
                            .collect::<Vec<_>>(),
                    )
                })
                .as_bytes(),
            )
            .await?;
        Ok(self)
    }

    pub async fn command0(&mut self, cmd: impl AsRef<str>) -> Result<&mut Self> {
        self.command(cmd, &[] as &[&str]).await
    }

    pub async fn environment<'a, F: Future<Output = Result<&'a mut Self>>>(
        &'a mut self,
        env: impl AsRef<str>,
        f: impl FnOnce(&'a mut Self) -> F,
    ) -> Result<&'a mut Self> {
        self.command("begin", [env.as_ref()]).await?;
        f(self).await?.command("end", [env]).await
    }
}
