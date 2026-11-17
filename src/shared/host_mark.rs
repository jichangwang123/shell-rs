use gpui_kit::component::{ActiveTheme as _, Icon, Sizable, Size, h_flex, tooltip::Tooltip};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::CatalogIcon;
use crate::host::{ConnectionState, HostOs, ProtocolKind};
use crate::i18n::t;

/// The monochrome mark for a host's connection type, shared by the host tree,
/// start page, and remote terminal tabs.
#[derive(IntoElement)]
pub struct HostMark {
    id: ElementId,
    address: SharedString,
    os: Option<HostOs>,
    protocol: ProtocolKind,
    /// The connection the mark stands for on a tab, when it has one.
    state: Option<ConnectionState>,
    size: Size,
    tooltip: bool,
}

impl HostMark {
    /// `address` picks the colour of a mark that has none of its own: the
    /// host's address, or a serial host's device.
    pub fn new(
        id: impl Into<ElementId>,
        _name: impl Into<SharedString>,
        address: impl Into<SharedString>,
        os: Option<HostOs>,
    ) -> Self {
        Self {
            id: id.into(),
            address: address.into(),
            os,
            protocol: ProtocolKind::Ssh,
            state: None,
            size: Size::Medium,
            tooltip: true,
        }
    }

    /// The mark of a Telnet or serial host, whatever `os` says: the name's
    /// character, which the tooltip calls Telnet or a serial port.
    pub fn protocol(mut self, protocol: ProtocolKind) -> Self {
        self.protocol = protocol;
        self
    }

    /// The state of the connection a tab's mark stands for. Anything but
    /// connected draws the mark grey, glyph and all, and says so.
    pub fn connection_state(mut self, state: ConnectionState) -> Self {
        self.state = Some(state);
        self
    }

    /// No tooltip of its own when the containing row supplies one.
    pub fn without_tooltip(mut self) -> Self {
        self.tooltip = false;
        self
    }
}

impl Sizable for HostMark {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for HostMark {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let compact = matches!(self.size, Size::XSmall | Size::Small);
        let (icon, description) = match self.os.filter(|_| self.protocol.is_ssh()) {
            Some(os) => (protocol_icon(self.protocol, &self.address), SharedString::from(os.label())),
            None => (
                protocol_icon(self.protocol, &self.address),
                match self.protocol {
                    ProtocolKind::Ssh => t!("shared.host_mark.unknown_os"),
                    ProtocolKind::Telnet => t!("shared.host_mark.telnet"),
                    ProtocolKind::Serial => t!("shared.host_mark.serial"),
                    ProtocolKind::Shell => t!("host.protocol.shell"),
                },
            ),
        };
        let offline = self.state.filter(|state| !state.is_connected());
        let description = match offline {
            Some(state) => t!(
                "shared.host_mark.with_state",
                mark = description,
                state = state.label()
            ),
            None => description,
        };
        let tooltip = description.clone();
        h_flex()
            .id(self.id)
            .test_support()
            .aria_label(description)
            .when(self.tooltip, |badge| {
                badge.tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            })
            .flex_shrink_0()
            .map(|badge| {
                if compact {
                    badge.size_5()
                } else {
                    badge.size_8()
                }
            })
            .justify_center()
            .text_color(if offline.is_some() {
                cx.theme().muted_foreground
            } else {
                cx.theme().foreground
            })
            .child(if compact {
                Icon::new(icon).size_3()
            } else {
                Icon::new(icon).size_5()
            })
    }
}

fn protocol_icon(protocol: ProtocolKind, address: &str) -> CatalogIcon {
    match protocol {
        ProtocolKind::Ssh => CatalogIcon::Server,
        ProtocolKind::Telnet => CatalogIcon::Globe,
        ProtocolKind::Serial => CatalogIcon::Cable,
        ProtocolKind::Shell if is_wsl_command(address) => CatalogIcon::SquareTerminal,
        ProtocolKind::Shell => CatalogIcon::Terminal,
    }
}

fn is_wsl_command(command: &str) -> bool {
    let command = command.trim_start();
    let executable = if let Some(quoted) = command.strip_prefix('"') {
        quoted.split('"').next().unwrap_or(quoted)
    } else if let Some(quoted) = command.strip_prefix('\'') {
        quoted.split('\'').next().unwrap_or(quoted)
    } else {
        command.split_whitespace().next().unwrap_or_default()
    };
    let executable = executable.rsplit(['\\', '/']).next().unwrap_or(executable);
    executable.eq_ignore_ascii_case("wsl") || executable.eq_ignore_ascii_case("wsl.exe")
}

#[cfg(test)]
mod tests {
    use super::{is_wsl_command, protocol_icon};
    use crate::app::CatalogIcon;
    use crate::host::ProtocolKind;

    #[test]
    fn connection_protocols_have_fixed_glyphs() {
        assert_eq!(protocol_icon(ProtocolKind::Ssh, "host"), CatalogIcon::Server);
        assert_eq!(
            protocol_icon(ProtocolKind::Telnet, "host"),
            CatalogIcon::Globe
        );
        assert_eq!(
            protocol_icon(ProtocolKind::Serial, "COM1"),
            CatalogIcon::Cable
        );
        assert_eq!(
            protocol_icon(ProtocolKind::Shell, "pwsh.exe"),
            CatalogIcon::Terminal
        );
        assert_eq!(
            protocol_icon(ProtocolKind::Shell, "wsl.exe -d Ubuntu"),
            CatalogIcon::SquareTerminal
        );
    }

    #[test]
    fn recognizes_wsl_executables_and_quoted_paths() {
        assert!(is_wsl_command("wsl.exe -d Ubuntu"));
        assert!(is_wsl_command("C:\\Windows\\System32\\wsl.exe -d Ubuntu"));
        assert!(is_wsl_command("\"C:\\Program Files\\WSL\\wsl.exe\" -d Ubuntu"));
        assert!(!is_wsl_command("powershell.exe"));
    }
}
