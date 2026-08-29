use crate::error::{AppError, AppResult};

const ERROR_MARKERS: &[&str] = &[
    "?",
    "NO DATA",
    "UNABLE TO CONNECT",
    "CAN ERROR",
    "BUS BUSY",
    "STOPPED",
    "BUFFER FULL",
    "OUT OF MEMORY",
    "UART RX OVERFLOW",
];

#[derive(Debug, Clone)]
pub struct StnReply {
    pub raw: String,
    pub lines: Vec<String>,
}

impl StnReply {
    pub fn parse(raw: String, command: &str) -> AppResult<Self> {
        let lines: Vec<String> = raw
            .replace('\r', "\n")
            .split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != ">")
            .filter(|line| !line.eq_ignore_ascii_case(command))
            .map(ToOwned::to_owned)
            .collect();

        if let Some(marker) = lines.iter().find(|line| {
            ERROR_MARKERS
                .iter()
                .any(|marker| line.eq_ignore_ascii_case(marker))
        }) {
            return Err(AppError::adapter(format!(
                "STN odmítl příkaz {command}: {marker}"
            )));
        }

        Ok(Self { raw, lines })
    }

    pub fn require_ok(&self, command: &str) -> AppResult<()> {
        if self
            .lines
            .iter()
            .any(|line| line.eq_ignore_ascii_case("OK"))
        {
            Ok(())
        } else {
            Err(AppError::adapter(format!(
                "Příkaz {command} nevrátil OK: {}",
                self.lines.join(" | ")
            )))
        }
    }

    pub fn first_value(&self, command: &str) -> AppResult<String> {
        self.lines
            .iter()
            .find(|line| {
                !line.eq_ignore_ascii_case("OK") && !line.eq_ignore_ascii_case("SEARCHING...")
            })
            .cloned()
            .ok_or_else(|| {
                AppError::adapter(format!("Příkaz {command} nevrátil očekávanou hodnotu."))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::StnReply;

    #[test]
    fn removes_echo_and_prompt() {
        let reply = StnReply::parse("STI\rSTN2232 v5.10.3\r>".into(), "STI").unwrap();
        assert_eq!(reply.lines, vec!["STN2232 v5.10.3"]);
    }

    #[test]
    fn recognizes_adapter_errors() {
        let error = StnReply::parse("NO DATA\r>".into(), "22F187").unwrap_err();
        assert_eq!(error.code, "ADAPTER_ERROR");
    }
}
