use ayaka_runtime::anyhow::Result;
use std::future::Future;
use tokio::{fs::File, io::AsyncWriteExt};

pub struct LaTeXWriter {
    file: File,
    ident: usize,
}

impl LaTeXWriter {
    pub fn new(file: File) -> Self {
        Self { file, ident: 0 }
    }

    async fn write_spaces(&mut self) -> Result<&mut Self> {
        self.file
            .write_all(
                &std::iter::repeat(b' ')
                    .take(self.ident * 4)
                    .collect::<Vec<_>>(),
            )
            .await?;
        Ok(self)
    }

    pub async fn write(&mut self, text: impl AsRef<str>) -> Result<&mut Self> {
        self.write_spaces().await?;
        self.file.write_all(text.as_ref().as_bytes()).await?;
        Ok(self)
    }

    async fn command_impl(
        &mut self,
        cmd: impl AsRef<str>,
        pre_attr: Option<impl AsRef<str>>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
        post_attr: Option<impl AsRef<str>>,
    ) -> Result<&mut Self> {
        self.write_spaces().await?;
        self.file
            .write_all(format!("\\{}", cmd.as_ref()).as_bytes())
            .await?;
        if let Some(attr) = pre_attr {
            self.file
                .write_all(format!("[{}]", attr.as_ref()).as_bytes())
                .await?;
        }
        let args = args
            .into_iter()
            .map(|arg| format!("{{{}}}", arg.as_ref()))
            .flat_map(|s| s.into_bytes())
            .collect::<Vec<_>>();
        self.file.write_all(&args).await?;
        if let Some(attr) = post_attr {
            self.file
                .write_all(format!("[{}]", attr.as_ref()).as_bytes())
                .await?;
        }
        self.file.write_all(&[b'\n']).await?;
        Ok(self)
    }

    pub async fn command(
        &mut self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<&mut Self> {
        self.command_impl(cmd, None::<&str>, args, None::<&str>)
            .await
    }

    pub async fn command0(&mut self, cmd: impl AsRef<str>) -> Result<&mut Self> {
        self.command(cmd, &[] as &[&str]).await
    }

    pub async fn command_attr(
        &mut self,
        cmd: impl AsRef<str>,
        attr: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<&mut Self> {
        self.command_impl(cmd, Some(attr), args, None::<&str>).await
    }

    async fn environment_impl<'a, F: Future<Output = Result<&'a mut Self>>>(
        &'a mut self,
        env: impl AsRef<str>,
        attr: Option<impl AsRef<str>>,
        f: impl FnOnce(&'a mut Self) -> F,
    ) -> Result<&'a mut Self> {
        self.command_impl("begin", None::<&str>, [env.as_ref()], attr)
            .await?;
        self.ident += 1;
        let this = f(self).await?;
        this.ident -= 1;
        this.command("end", [env]).await
    }

    pub async fn environment<'a, F: Future<Output = Result<&'a mut Self>>>(
        &'a mut self,
        env: impl AsRef<str>,
        f: impl FnOnce(&'a mut Self) -> F,
    ) -> Result<&'a mut Self> {
        self.environment_impl(env, None::<&str>, f).await
    }

    pub async fn environment_attr<'a, F: Future<Output = Result<&'a mut Self>>>(
        &'a mut self,
        env: impl AsRef<str>,
        attr: impl AsRef<str>,
        f: impl FnOnce(&'a mut Self) -> F,
    ) -> Result<&'a mut Self> {
        self.environment_impl(env, Some(attr), f).await
    }
}
