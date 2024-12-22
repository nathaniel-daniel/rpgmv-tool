use anyhow::ensure;
use anyhow::Context;

#[derive(Debug)]
pub struct ParamReader<'a> {
    command: &'a rpgmv_types::EventCommand,
}

impl<'a> ParamReader<'a> {
    pub fn new(command: &'a rpgmv_types::EventCommand) -> Self {
        Self { command }
    }

    pub fn ensure_len_is(&self, expected_len: usize) -> anyhow::Result<()> {
        let actual_len = self.command.parameters.len();
        ensure!(
            actual_len == expected_len,
            "expected {expected_len} parameters, but got {actual_len}"
        );

        Ok(())
    }

    pub fn read_at<T>(&self, index: usize, parameter_name: &str) -> anyhow::Result<T>
    where
        T: ParamReaderOutput,
    {
        self.read_at_inner(index)
            .with_context(|| format!("failed to read parameter \"{parameter_name}\""))
    }

    fn read_at_inner<T>(&self, index: usize) -> anyhow::Result<T>
    where
        T: ParamReaderOutput,
    {
        let value = self
            .command
            .parameters
            .get(index)
            .with_context(|| format!("parameter index {index} is out of range"))?;

        T::from_param(value)
    }
}

pub trait ParamReaderOutput: Sized {
    fn from_param(value: &serde_json::Value) -> anyhow::Result<Self>;
}

impl ParamReaderOutput for String {
    fn from_param(value: &serde_json::Value) -> anyhow::Result<Self> {
        let value = value.as_str().context("not a string")?.to_string();

        Ok(value)
    }
}

impl ParamReaderOutput for i64 {
    fn from_param(value: &serde_json::Value) -> anyhow::Result<Self> {
        value.as_i64().context("not an i64")
    }
}

impl ParamReaderOutput for u32 {
    fn from_param(value: &serde_json::Value) -> anyhow::Result<Self> {
        let value = i64::from_param(value)?;
        u32::try_from(value).context("i64 value out of range for u32")
    }
}
