use eframe::egui::Color32;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    DarkMode,
    Retro,
    MacOsX,
    Windows98,
    WindowsXp,
    Matrix,
}

impl ColorScheme {
    pub const ALL: &'static [ColorScheme] = &[
        ColorScheme::DarkMode,
        ColorScheme::Retro,
        ColorScheme::MacOsX,
        ColorScheme::Windows98,
        ColorScheme::WindowsXp,
        ColorScheme::Matrix,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ColorScheme::DarkMode => "Dark Mode",
            ColorScheme::Retro => "SNES",
            ColorScheme::MacOsX => "Mac OS X",
            ColorScheme::Windows98 => "Windows 98",
            ColorScheme::WindowsXp => "Windows XP",
            ColorScheme::Matrix => "Matrix",
        }
    }

    pub fn is_dark(self) -> bool {
        match self {
            ColorScheme::DarkMode => true,
            ColorScheme::Retro => true,
            ColorScheme::MacOsX => false,
            ColorScheme::Windows98 => false,
            ColorScheme::WindowsXp => false,
            ColorScheme::Matrix => true,
        }
    }

    pub fn theme(self) -> ThemeColors {
        match self {
            ColorScheme::DarkMode => ThemeColors {
                dir_base: [40, 70, 140],
                dir_range: [30, 40, 80],
                file_base: [50, 120, 50],
                file_range: [50, 80, 50],
                dir_border: Color32::from_rgb(200, 200, 255),
                file_border: Color32::from_rgb(200, 255, 200),
                text_primary: Color32::WHITE,
                text_secondary: Color32::from_rgb(220, 220, 220),
                indicator: Color32::WHITE,
                hover_boost: 30,
            },
            ColorScheme::Retro => ThemeColors {
                // SNES-inspired: purple/lavender dirs, muted blue-grey files
                dir_base: [75, 50, 130],
                dir_range: [40, 30, 50],
                file_base: [80, 80, 110],
                file_range: [35, 35, 40],
                dir_border: Color32::from_rgb(150, 120, 200),
                file_border: Color32::from_rgb(140, 140, 170),
                text_primary: Color32::from_rgb(230, 230, 240),
                text_secondary: Color32::from_rgb(180, 175, 200),
                indicator: Color32::from_rgb(200, 180, 255),
                hover_boost: 25,
            },
            ColorScheme::MacOsX => ThemeColors {
                dir_base: [60, 130, 200],
                dir_range: [30, 30, 40],
                file_base: [160, 170, 180],
                file_range: [30, 25, 25],
                dir_border: Color32::from_rgb(100, 160, 220),
                file_border: Color32::from_rgb(180, 185, 190),
                text_primary: Color32::from_rgb(30, 30, 30),
                text_secondary: Color32::from_rgb(80, 80, 80),
                indicator: Color32::from_rgb(30, 30, 30),
                hover_boost: 20,
            },
            ColorScheme::Windows98 => ThemeColors {
                dir_base: [0, 0, 128],
                dir_range: [20, 20, 40],
                file_base: [0, 128, 128],
                file_range: [30, 30, 30],
                dir_border: Color32::from_rgb(255, 255, 255),
                file_border: Color32::from_rgb(223, 223, 223),
                text_primary: Color32::WHITE,
                text_secondary: Color32::from_rgb(192, 192, 192),
                indicator: Color32::from_rgb(255, 255, 0),
                hover_boost: 35,
            },
            ColorScheme::WindowsXp => ThemeColors {
                dir_base: [0, 78, 152],
                dir_range: [30, 30, 40],
                file_base: [55, 126, 34],
                file_range: [30, 40, 30],
                dir_border: Color32::from_rgb(60, 150, 220),
                file_border: Color32::from_rgb(100, 180, 80),
                text_primary: Color32::WHITE,
                text_secondary: Color32::from_rgb(220, 230, 240),
                indicator: Color32::WHITE,
                hover_boost: 25,
            },
            ColorScheme::Matrix => ThemeColors {
                dir_base: [0, 80, 0],
                dir_range: [10, 60, 10],
                file_base: [0, 50, 0],
                file_range: [10, 40, 10],
                dir_border: Color32::from_rgb(0, 200, 0),
                file_border: Color32::from_rgb(0, 150, 0),
                text_primary: Color32::from_rgb(0, 255, 0),
                text_secondary: Color32::from_rgb(0, 200, 0),
                indicator: Color32::from_rgb(0, 255, 0),
                hover_boost: 20,
            },
        }
    }
}

pub struct ThemeColors {
    pub dir_base: [u8; 3],
    pub dir_range: [u8; 3],
    pub file_base: [u8; 3],
    pub file_range: [u8; 3],
    pub dir_border: Color32,
    pub file_border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub indicator: Color32,
    pub hover_boost: u8,
}

impl ThemeColors {
    pub fn color_for_node(&self, name: &str, is_dir: bool) -> Color32 {
        let hash_input = if is_dir {
            name.to_string()
        } else {
            name.rsplit('.').next().unwrap_or("").to_string()
        };
        let mut hasher = DefaultHasher::new();
        hash_input.hash(&mut hasher);
        let h = hasher.finish();

        let (base, range) = if is_dir {
            (&self.dir_base, &self.dir_range)
        } else {
            (&self.file_base, &self.file_range)
        };

        Color32::from_rgb(
            base[0].saturating_add((h % range[0].max(1) as u64) as u8),
            base[1].saturating_add(((h >> 8) % range[1].max(1) as u64) as u8),
            base[2].saturating_add(((h >> 16) % range[2].max(1) as u64) as u8),
        )
    }

    pub fn hover_color(&self, base: Color32) -> Color32 {
        Color32::from_rgb(
            base.r().saturating_add(self.hover_boost),
            base.g().saturating_add(self.hover_boost),
            base.b().saturating_add(self.hover_boost),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_schemes_have_names() {
        for &scheme in ColorScheme::ALL {
            assert!(!scheme.name().is_empty());
        }
    }

    #[test]
    fn all_schemes_count() {
        assert_eq!(ColorScheme::ALL.len(), 6);
    }

    #[test]
    fn dark_schemes_identified() {
        assert!(ColorScheme::DarkMode.is_dark());
        assert!(ColorScheme::Retro.is_dark());
        assert!(ColorScheme::Matrix.is_dark());
    }

    #[test]
    fn light_schemes_identified() {
        assert!(!ColorScheme::MacOsX.is_dark());
        assert!(!ColorScheme::Windows98.is_dark());
        assert!(!ColorScheme::WindowsXp.is_dark());
    }

    #[test]
    fn color_for_dir_uses_dir_palette() {
        let theme = ColorScheme::DarkMode.theme();
        let color = theme.color_for_node("mydir", true);
        // Should be in dir_base range (blue-ish for dark mode)
        assert!(color.r() >= theme.dir_base[0]);
        assert!(color.g() >= theme.dir_base[1]);
        assert!(color.b() >= theme.dir_base[2]);
    }

    #[test]
    fn color_for_file_uses_file_palette() {
        let theme = ColorScheme::DarkMode.theme();
        let color = theme.color_for_node("test.rs", false);
        assert!(color.r() >= theme.file_base[0]);
        assert!(color.g() >= theme.file_base[1]);
        assert!(color.b() >= theme.file_base[2]);
    }

    #[test]
    fn same_extension_same_color() {
        let theme = ColorScheme::DarkMode.theme();
        let c1 = theme.color_for_node("foo.rs", false);
        let c2 = theme.color_for_node("bar.rs", false);
        assert_eq!(c1, c2);
    }

    #[test]
    fn different_extension_may_differ() {
        let theme = ColorScheme::DarkMode.theme();
        let c1 = theme.color_for_node("foo.rs", false);
        let c2 = theme.color_for_node("foo.txt", false);
        // Not guaranteed to differ due to hash collisions, but verify both produce colors
        assert_ne!((c1.r(), c1.g(), c1.b()), (0, 0, 0));
        assert_ne!((c2.r(), c2.g(), c2.b()), (0, 0, 0));
    }

    #[test]
    fn dir_color_deterministic() {
        let theme = ColorScheme::Matrix.theme();
        let c1 = theme.color_for_node("src", true);
        let c2 = theme.color_for_node("src", true);
        assert_eq!(c1, c2);
    }

    #[test]
    fn hover_color_brighter() {
        let theme = ColorScheme::DarkMode.theme();
        let base = Color32::from_rgb(100, 100, 100);
        let hover = theme.hover_color(base);
        assert!(hover.r() > base.r());
        assert!(hover.g() > base.g());
        assert!(hover.b() > base.b());
    }

    #[test]
    fn hover_color_saturates_at_255() {
        let theme = ColorScheme::Windows98.theme(); // hover_boost = 35
        let base = Color32::from_rgb(240, 240, 240);
        let hover = theme.hover_color(base);
        assert_eq!(hover.r(), 255);
        assert_eq!(hover.g(), 255);
        assert_eq!(hover.b(), 255);
    }

    #[test]
    fn each_scheme_produces_valid_theme() {
        for &scheme in ColorScheme::ALL {
            let theme = scheme.theme();
            // Verify hover_boost is reasonable
            assert!(theme.hover_boost > 0);
            assert!(theme.hover_boost <= 50);
            // Verify colors are non-zero for at least some channels
            let has_color = theme.dir_base.iter().any(|&c| c > 0)
                || theme.file_base.iter().any(|&c| c > 0);
            assert!(has_color, "Scheme {} has no color", scheme.name());
        }
    }
}
