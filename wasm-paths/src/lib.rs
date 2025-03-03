use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Once;

use glam::{DAffine2, DVec2};
use rustybuzz as hb; // alias for harfbuzz
use wasm_bindgen::prelude::*;

macro_rules! log {
    ($($arg:tt)*) => ({
        let msg = format!($($arg)*);
        web_sys::console::log_1(&msg.into());
    });
}

#[derive(Default)]
struct AppState<'a> {
    fonts: FontRegistry<'a>,
    inputs: Vec<Input>,
}

struct InputTransform {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    size: usize,
}

#[wasm_bindgen]
pub fn get_paths(x: i32, y: i32, w: i32, h: i32, size: usize, input: usize) -> Vec<String> {
    let state = app_state();
    let input_transform = InputTransform { x, y, w, h, size };

    state.resolve_input(&input_transform, input)
}

struct StaticFont<'a> {
    raw_data: &'a [u8],
    face: hb::Face<'a>,
}

struct VariableFont<'a> {
    raw_data: &'a [u8],
    face: hb::Face<'a>,
}

enum Font<'a> {
    StaticFont(StaticFont<'a>),
    VariableFont(VariableFont<'a>),
}

impl<'a> Font<'a> {}

type FontId = String;
type FontRegistry<'a> = HashMap<FontId, Font<'a>>;

const FONT_WEIGHTS: [&'static str; 6] = [
    "light",
    "light-italic",
    "normal",
    "normal-italic",
    "bold",
    "bold-italic",
];

const GLOBAL_FALLBACK_FONT: &'static str = "pt";

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum FontWeight {
    Light,
    LightItalic,
    Normal,
    NormalItalic,
    Bold,
    BoldItalic,
}

struct FontStyle {
    size_scale: f32,
    color: u32,
    weight: FontWeight,
    underline: bool,
    strikethrough: bool,
}

struct TextSpanStyle {
    font_id: FontId,
    font_style: FontStyle,
}

struct TextSpan {
    start_index: usize,
    end_index: usize,
    style: TextSpanStyle,
}

const FONT_DATA: [&'static [u8]; 5] = [
    include_bytes!("../fonts/PTSerif-Regular.ttf"),
    include_bytes!("../fonts/SeoulNamsanvert.otf"),
    include_bytes!("../fonts/Roboto-VariableFont_wdth,wght.ttf"),
    include_bytes!("../fonts/Roboto-Italic-VariableFont_wdth,wght.ttf"),
    include_bytes!("../fonts/NotoSansHebrew-VariableFont_wdth,wght.ttf"),
];

struct Input {
    text: String,
    spans: Vec<TextSpan>,
    paragraphs_fonts: Vec<FontId>,
    fallback_font: FontId,
}

impl<'a> AppState<'a> {
    fn new() -> AppState<'a> {
        let mut fonts = HashMap::<FontId, Font<'a>>::new();

        fonts.insert(
            GLOBAL_FALLBACK_FONT.into(),
            Font::StaticFont(StaticFont {
                raw_data: FONT_DATA[0],
                face: hb::Face::from_slice(FONT_DATA[0], 0).unwrap(),
            }),
        );
        fonts.insert(
            "seoul".into(),
            Font::StaticFont(StaticFont {
                raw_data: FONT_DATA[1],
                face: hb::Face::from_slice(FONT_DATA[1], 0).unwrap(),
            }),
        );
        fonts.insert(
            "roboto".into(),
            Font::VariableFont(VariableFont {
                raw_data: FONT_DATA[2],
                face: hb::Face::from_slice(FONT_DATA[2], 0).unwrap(),
            }),
        );
        fonts.insert(
            "roboto-italic".into(),
            Font::VariableFont(VariableFont {
                raw_data: FONT_DATA[3],
                face: hb::Face::from_slice(FONT_DATA[3], 0).unwrap(),
            }),
        );
        fonts.insert(
            "noto".into(),
            Font::VariableFont(VariableFont {
                raw_data: FONT_DATA[4],
                face: hb::Face::from_slice(FONT_DATA[4], 0).unwrap(),
            }),
        );

        let inputs = vec![
            Input {
                text: "아무도 자의적인 체포, 구금 또는 추방을 당하지 않아야 합니다. 모든 사람은 자신의 권리와 의무, 그리고 자신에게 제기된 형사 혐의를 결정함에 있어 독립적이고 공정한 재판소에 의해 평등하게 공정하고 공개적인 심리를 받을 권리를 갖습니다. 아무도 자신의 사생활, 가족, 가정 또는 서신에 대한 자의적인 간섭이나 명예와 평판에 대한 공격을 받아서는 안 됩니다. 모든 사람은 그러한 간섭이나 공격으로부터 법의 보호를 받을 권리를 갖습니다.".into(),
                spans: vec![],
                paragraphs_fonts: vec!["seoul".into()],
                fallback_font: "seoul".into(),
            },
            Input {
                text: "איש לא יהיה נתון למעצר, מעצר שרירותי או גירוש. לכל אדם הזכות לשוויון מלא למשפט הוגן ופומבי בפני בית דין עצמאי ובלתי משוחד, לצורך הכרעה בזכויותיו וחובותיו ובכל אישום פלילי המופנה נגדו. איש לא יהיה נתון להתערבות שרירותית בפרטיותו, במשפחתו, בביתו או בהתכתבויותיו, ולא לפגיעות בכבודו או בשמו הטוב. לכל אדם הזכות להגנת החוק מפני התערבויות או פגיעות כאלה.".into(),
                spans: vec![],
                paragraphs_fonts: vec!["noto".into()],
                fallback_font: "noto".into(),
            },
            Input {
                text: "Nul ne sera soumis à une arrestation, une détention ou un exil arbitraires. Toute personne a droit, en pleine égalité, à ce que sa cause soit entendue équitablement et publiquement par un tribunal indépendant et impartial, qui décidera de ses droits et obligations ainsi que du bien-fondé de toute accusation en matière pénale portée contre elle. Nul ne sera l'objet d'immixtions arbitraires dans sa vie privée, sa famille, son domicile ou sa correspondance, ni d'atteintes à son honneur et à sa réputation. Toute personne a droit à la protection de la loi contre de telles immixtions ou de telles atteintes.".into(),
                spans: vec![],
                paragraphs_fonts: vec!["pt".into()],
                fallback_font: "pt".into(),
            },
            Input {
                text: "Nul ne sera soumis à une arrestation, une détention ou un exil arbitraires. \n איש לא יהיה נתון להתערבות שרירותית בפרטיותו, במשפחתו, בביתו או בהתכתבויותיו, ולא לפגיעות בכבודו או בשמו הטוב \n Toute personne a droit à la protection de la loi contre de telles immixtions ou de telles atteintes.".into(),
                spans: vec![],
                paragraphs_fonts: vec!["roboto".into(), "noto".into(), "roboto".into()],
                fallback_font: "roboto".into(),
            }
        ];

        AppState::<'a> { fonts, inputs }
    }

    fn resolve_input(&self, input_transform: &InputTransform, input: usize) -> Vec<String> {
        use icu::properties::bidi::BidiClassAdapter;
        use icu::properties::maps;
        use unicode_bidi::BidiInfo;

        let adapter = BidiClassAdapter::new(maps::bidi_class());
        let bidi_info =
            BidiInfo::new_with_data_source(&adapter, self.inputs[input].text.as_ref(), None);

        let mut layout_paragraps =
            Vec::<(String, &Font, bool)>::with_capacity(bidi_info.paragraphs.len());

        for (i, paragraph) in bidi_info.paragraphs.iter().enumerate() {
            let line = paragraph.range.clone();
            let display_str: String = bidi_info.reorder_line(paragraph, line).into_owned();
            let is_rtl = paragraph.level.is_rtl();

            let mut font = self.fonts.get(&self.inputs[input].paragraphs_fonts[i]);
            if font.is_none() {
                log!(
                    "Can't draw text with font {} because it was not found! Using {} instead.",
                    self.inputs[input].paragraphs_fonts[i],
                    self.inputs[input].fallback_font,
                );
                font = self.fonts.get(&self.inputs[input].fallback_font);
                if font.is_none() {
                    log!(
                        "Can't draw text with font {} because it was not found! Using {} instead.",
                        self.inputs[input].fallback_font,
                        GLOBAL_FALLBACK_FONT
                    );
                }
            }
            let font = font.unwrap_or(self.fonts.get(GLOBAL_FALLBACK_FONT).unwrap());
            layout_paragraps.push((display_str, font, is_rtl));
        }

        self.perform_layout_on_paragraphs(input_transform, &layout_paragraps)
    }

    fn perform_layout_on_paragraphs(
        &self,
        input_transform: &InputTransform,
        paragraphs: &[(String, &Font, bool)],
    ) -> Vec<String> {
        const PAD: f64 = 4.0;
        let line_height = 1.25 * (input_transform.size as f64);
        let mut result = vec![];

        for (i, (text, font, is_rtl)) in paragraphs.iter().enumerate() {
            let font = *font;
            let mut baseline_point = DVec2::new(
                input_transform.x as f64 + PAD,
                input_transform.y as f64 + PAD + line_height * ((i + 1) as f64),
            );

            if *is_rtl {
                baseline_point.x = ((input_transform.x + input_transform.w) as f64) - PAD;
            }

            match font {
                Font::StaticFont(f) => {
                    let (glyphs, baseline) =
                        self.shape_static_text(text, f, input_transform, baseline_point);
                    baseline_point = baseline;
                    result.extend(glyphs);
                }
                Font::VariableFont(_f) => {
                    log!("Can't currently draw variable fonts!");
                }
            }
        }

        result
    }

    fn shape_static_text(
        &self,
        text: &str,
        font: &StaticFont,
        input_transform: &InputTransform,
        baseline_point: DVec2,
    ) -> (Vec<String>, DVec2) {
        let mut baseline_point = baseline_point;
        let mut result = vec![];
        let mut buffer = hb::UnicodeBuffer::new();

        buffer.push_str(text);
        buffer.guess_segment_properties();
        buffer.set_cluster_level(hb::BufferClusterLevel::MonotoneCharacters);

        if buffer.direction() == hb::Direction::TopToBottom {
            log!("Laying out vertical text is unsupported! Detected: TopToBottom, replacing with: LeftToRight");
            buffer.set_direction(hb::Direction::LeftToRight);
        }
        if buffer.direction() == hb::Direction::BottomToTop {
            log!("Laying out vertical text is unsupported! Detected: BottomToTop, replacing with: RightToLeft");
            buffer.set_direction(hb::Direction::RightToLeft);
        }

        let glyph_buffer = hb::shape(&font.face, &[], buffer);
        for (glyph, info) in glyph_buffer
            .glyph_positions()
            .iter()
            .zip(glyph_buffer.glyph_infos().iter())
        {
            let (advance_x, advance_y, offset_x, offset_y) = (
                glyph.x_advance,
                glyph.y_advance,
                glyph.x_offset,
                glyph.y_offset,
            );
            let glyph_id = hb::ttf_parser::GlyphId(info.glyph_id.try_into().unwrap());
            let offset = DVec2::new(offset_x as f64, offset_y as f64);
            let font_transform =
                Self::from_font_space_to_screen_space(&font.face, input_transform.size, offset);
            let glyph_transform = DAffine2::from_translation(baseline_point) * font_transform;
            let mut glyph_path = GlyphPath {
                svg_path_string: "".into(),
                transform: glyph_transform,
            };
            font.face.outline_glyph(glyph_id, &mut glyph_path);
            result.push(glyph_path.svg_path_string);

            let advance = DVec2::new(advance_x as f64, advance_y as f64);
            let advance = font_transform.transform_point2(advance);
            baseline_point += advance;
        }

        (result, baseline_point)
    }

    fn from_font_space_to_screen_space(
        face: &hb::Face,
        text_size: usize,
        offset: DVec2,
    ) -> DAffine2 {
        let units_per_em = face.units_per_em();
        let (ppem, upem) = (text_size as f64, units_per_em as f64);
        // `ppem` gives us the mapping between font units and screen pixels.
        // ppem stands for pixels per em.
        let to_px = ppem / upem;

        DAffine2::from_scale(DVec2::new(to_px, -to_px)) * DAffine2::from_translation(offset)
    }
}

#[allow(static_mut_refs)]
fn app_state() -> &'static mut AppState<'static> {
    static mut SINGLETON: MaybeUninit<AppState> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let singleton = AppState::new();
            SINGLETON.write(singleton);
        });

        SINGLETON.assume_init_mut()
    }
}

#[derive(Debug, Clone)]
struct GlyphPath {
    svg_path_string: String,
    transform: DAffine2,
}

impl hb::ttf_parser::OutlineBuilder for GlyphPath {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = DVec2::new(x as f64, y as f64);
        let to = self.transform.transform_point2(to);
        self.svg_path_string += &format!("M{} {} ", to.x, to.y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = DVec2::new(x as f64, y as f64);
        let to = self.transform.transform_point2(to);
        self.svg_path_string += &format!("L{} {} ", to.x, to.y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = DVec2::new(x1 as f64, y1 as f64);
        let p2 = DVec2::new(x as f64, y as f64);

        let p1 = self.transform.transform_point2(p1);
        let p2 = self.transform.transform_point2(p2);

        self.svg_path_string += &format!("Q{} {},{} {} ", p1.x, p1.y, p2.x, p2.y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p1 = DVec2::new(x1 as f64, y1 as f64);
        let p1 = self.transform.transform_point2(p1);
        let p2 = DVec2::new(x2 as f64, y2 as f64);
        let p2 = self.transform.transform_point2(p2);
        let p3 = DVec2::new(x as f64, y as f64);
        let p3 = self.transform.transform_point2(p3);

        self.svg_path_string += &format!("C{} {},{} {},{} {} ", p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
    }

    fn close(&mut self) {
        self.svg_path_string += &format!("Z ");
    }
}
