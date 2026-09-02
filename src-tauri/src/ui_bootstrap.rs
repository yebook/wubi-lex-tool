//! Fixed first-frame projection of durable UI preferences into the WebView.

use crate::config::{AppLocale, Density, ThemePreference, UiConfig};
use tauri::{Manager, Runtime, Theme, WebviewWindowBuilder};

const SCRIPT_OPEN: &str = "(()=>{if(window.top!==window.self){return;}const preference=\"";
const SCRIPT_DENSITY: &str = "\";const density=\"";
const SCRIPT_LOCALE: &str = "\";const locale=\"";
const SCRIPT_BODY: &str = r#"";const apply=()=>{const root=document.documentElement;if(!root){return false;}const dark=preference==="dark"||(preference==="system"&&window.matchMedia("(prefers-color-scheme: dark)").matches);root.setAttribute("data-theme",preference);root.setAttribute("data-density",density);root.setAttribute("lang",locale);root.classList.toggle("dark",dark);root.style.colorScheme=preference==="system"?"light dark":preference;return true;};if(apply()){return;}const observer=new MutationObserver(()=>{if(apply()){observer.disconnect();}});observer.observe(document,{childList:true});})();"#;

/// Closed projection used before frontend JavaScript starts.
#[derive(Debug)]
pub(crate) struct UiBootstrap {
    script: String,
    native_theme: Option<Theme>,
    background: Option<(u8, u8, u8, u8)>,
}

impl UiBootstrap {
    pub(crate) fn from_config(config: &UiConfig) -> Self {
        let theme = match config.theme {
            ThemePreference::System => "system",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        };
        let density = match config.density {
            Density::Standard => "standard",
            Density::Compact => "compact",
        };
        let locale = match config.locale {
            AppLocale::ZhCn => "zh-CN",
        };

        let mut script = String::with_capacity(
            SCRIPT_OPEN.len()
                + theme.len()
                + SCRIPT_DENSITY.len()
                + density.len()
                + SCRIPT_LOCALE.len()
                + locale.len()
                + SCRIPT_BODY.len(),
        );
        script.push_str(SCRIPT_OPEN);
        script.push_str(theme);
        script.push_str(SCRIPT_DENSITY);
        script.push_str(density);
        script.push_str(SCRIPT_LOCALE);
        script.push_str(locale);
        script.push_str(SCRIPT_BODY);

        let (native_theme, background) = match config.theme {
            ThemePreference::System => (None, None),
            ThemePreference::Light => (Some(Theme::Light), Some((255, 255, 255, 255))),
            ThemePreference::Dark => (Some(Theme::Dark), Some((22, 25, 29, 255))),
        };

        Self {
            script,
            native_theme,
            background,
        }
    }

    #[cfg(test)]
    fn script(&self) -> &str {
        &self.script
    }
}

pub(crate) fn apply_to_builder<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
    bootstrap: UiBootstrap,
) -> WebviewWindowBuilder<'a, R, M> {
    let mut builder = builder
        .initialization_script(bootstrap.script)
        .theme(bootstrap.native_theme);
    if let Some(background) = bootstrap.background {
        builder = builder.background_color(background.into());
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::UiBootstrap;
    use crate::config::{AppLocale, Density, ThemePreference, UiConfig};
    use tauri::Theme;

    fn config(theme: ThemePreference, density: Density) -> UiConfig {
        UiConfig {
            theme,
            density,
            locale: AppLocale::ZhCn,
            sidebar_collapsed: true,
            onboarding_version: u32::MAX,
        }
    }

    #[test]
    fn default_projection_is_system_standard_simplified_chinese() {
        let bootstrap = UiBootstrap::from_config(&UiConfig::default());

        assert!(bootstrap.script().contains("preference=\"system\""));
        assert!(bootstrap.script().contains("density=\"standard\""));
        assert!(bootstrap.script().contains("locale=\"zh-CN\""));
        assert_eq!(bootstrap.native_theme, None);
        assert_eq!(bootstrap.background, None);
    }

    #[test]
    fn every_theme_and_density_use_only_closed_literals() {
        for (theme, literal, native, background) in [
            (ThemePreference::System, "system", None, None),
            (
                ThemePreference::Light,
                "light",
                Some(Theme::Light),
                Some((255, 255, 255, 255)),
            ),
            (
                ThemePreference::Dark,
                "dark",
                Some(Theme::Dark),
                Some((22, 25, 29, 255)),
            ),
        ] {
            for (density, density_literal) in [
                (Density::Standard, "standard"),
                (Density::Compact, "compact"),
            ] {
                let bootstrap = UiBootstrap::from_config(&config(theme, density));
                assert!(
                    bootstrap
                        .script()
                        .contains(&format!("preference=\"{literal}\""))
                );
                assert!(
                    bootstrap
                        .script()
                        .contains(&format!("density=\"{density_literal}\""))
                );
                assert_eq!(bootstrap.native_theme, native);
                assert_eq!(bootstrap.background, background);
                assert!(!bootstrap.script().contains("4294967295"));
                assert!(!bootstrap.script().contains("sidebar"));
            }
        }
    }

    #[test]
    fn script_guards_subframes_and_handles_a_missing_document_element() {
        let script = UiBootstrap::from_config(&UiConfig::default()).script;

        assert!(script.contains("window.top!==window.self"));
        assert!(script.contains("if(!root){return false;}"));
        assert!(script.contains("new MutationObserver"));
        assert!(script.contains("observer.disconnect()"));
        assert!(script.contains("classList.toggle(\"dark\",dark)"));
        assert!(script.contains("prefers-color-scheme: dark"));
        assert!(script.contains("data-theme"));
        assert!(script.contains("data-density"));
        assert!(script.contains("colorScheme"));
    }
}
