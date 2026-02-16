use serde::Serialize;
use anyhow::Result;
pub use ratatui::style::Color;

pub trait MachineOutput: Serialize {
    fn to_machine_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }
}

pub trait Renderable: MachineOutput {
    fn render_human(&self) -> Result<()>;
}

#[derive(Clone, Copy)]
pub enum OutputMode {
    Human,
    Machine,
}

impl OutputMode {
    pub fn render<T: Renderable>(&self, data: T) -> Result<()> {
        match self {
            OutputMode::Human => data.render_human(),
            OutputMode::Machine => {
                println!("{}", data.to_machine_string()?);
                Ok(())
            }
        }
    }
}

pub mod colors {
    use super::Color;
    pub const ACCENT_BLUE: Color = Color::Rgb(77, 163, 255);
    pub const ACCENT_GREEN: Color = Color::Rgb(110, 235, 131);
    pub const ACCENT_YELLOW: Color = Color::Rgb(242, 201, 76);
    pub const ACCENT_RED: Color = Color::Rgb(235, 87, 87);
    pub const ACCENT_MAGENTA: Color = Color::Rgb(187, 107, 217);
    pub const ACCENT_CYAN: Color = Color::Rgb(86, 204, 242);
    pub const BG_DARK: Color = Color::Rgb(13, 13, 13);
    pub const PANEL_DARK: Color = Color::Rgb(26, 26, 26);
    pub const TEXT_LIGHT: Color = Color::Rgb(237, 237, 237);
    pub const TEXT_DIM: Color = Color::Rgb(150, 150, 150);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    impl MachineOutput for TestData {}

    #[test]
    fn test_machine_output() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        let yaml = data.to_machine_string().unwrap();
        assert!(yaml.contains("name: test"));
        assert!(yaml.contains("value: 42"));
    }
}
