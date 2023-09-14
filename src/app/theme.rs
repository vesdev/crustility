pub fn horizon_dark() -> egui::Visuals {
    let dark_pink = color(0xe95378);
    let orange = color(0xe4a88a);
    let text = color(0xd5d8da);
    let warn = color(0xe4a88a);
    let error = color(0xe95378);
    let url = color(0xe95378);
    let selection = color(0x353747);
    let background = color(0x1c1e26);
    let background_light = color(0x353747);
    let background_dark = color(0x16161c);
    let stroke = egui::Stroke::new(0., egui::Color32::TRANSPARENT);
    let stroke_light = egui::Stroke::new(1., color(0x2f3138));
    let stroke_orange = egui::Stroke::new(1., orange);
    let _stroke_dark = egui::Stroke::new(1., background_dark);
    let rounding = egui::Rounding::default();
    let shadow = egui::epaint::Shadow {
        extrusion: 5.,
        color: background_dark,
    };

    egui::Visuals {
        dark_mode: true,
        override_text_color: Some(text),
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: background_dark,
                weak_bg_fill: background_light,
                bg_stroke: stroke_light,
                rounding,
                fg_stroke: stroke,
                expansion: 1.,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: background_dark,
                weak_bg_fill: background_light,
                bg_stroke: stroke,
                rounding,
                fg_stroke: stroke_orange,
                expansion: 1.,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: background_light,
                weak_bg_fill: background_dark,
                bg_stroke: stroke_light,
                rounding,
                fg_stroke: stroke_orange,
                expansion: 1.,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: background,
                weak_bg_fill: dark_pink,
                bg_stroke: stroke,
                rounding,
                fg_stroke: stroke_orange,
                expansion: 1.,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: background,
                weak_bg_fill: background_dark,
                bg_stroke: stroke,
                rounding,
                fg_stroke: stroke,
                expansion: 1.,
            },
        },
        selection: egui::style::Selection {
            bg_fill: selection,
            stroke,
        },
        hyperlink_color: url,
        faint_bg_color: background_dark,
        extreme_bg_color: background_dark,
        code_bg_color: background,
        warn_fg_color: warn,
        error_fg_color: error,
        window_rounding: rounding,
        window_shadow: shadow,
        window_fill: background,
        window_stroke: stroke,
        menu_rounding: rounding,
        panel_fill: background,
        popup_shadow: shadow,
        resize_corner_size: 3.,
        text_cursor: stroke,
        text_cursor_preview: false,
        clip_rect_margin: 5.,
        button_frame: true,
        collapsing_header_frame: false,
        indent_has_left_vline: true,
        striped: true,
        slider_trailing_fill: false,
        interact_cursor: None,
    }
}

fn color(c: u32) -> egui::Color32 {
    egui::Color32::from_rgb(
        ((c & 0xFF0000) >> 16) as u8,
        ((c & 0x00FF00) >> 8) as u8,
        (c & 0x0000FF) as u8,
    )
}
