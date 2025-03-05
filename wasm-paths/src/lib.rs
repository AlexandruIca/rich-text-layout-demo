use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Once;

use glam::{DAffine2, DVec2};
use rustybuzz as hb; // alias for harfbuzz
use wasm_bindgen::prelude::*;

macro_rules! log {
    ($($arg:tt)*) => ({
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!($($arg)*);
            web_sys::console::log_1(&msg.into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!($($arg)*);
        }
    });
}

#[derive(Default)]
struct AppState<'a> {
    fonts: FontRegistry<'a>,
    inputs: Vec<Input>,
    last_input: usize,
    last_text_size: usize,
    already_performed_layout: bool,
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

struct Font<'a> {
    raw_data: &'a [u8],
    face: hb::Face<'a>,
}

impl<'a> Font<'a> {}

type FontId = String;
type FontRegistry<'a> = HashMap<FontId, Font<'a>>;

const GLOBAL_FALLBACK_FONT: &'static str = "pt";

const FONT_DATA: [&'static [u8]; 5] = [
    include_bytes!("../fonts/PTSerif-Regular.ttf"),
    include_bytes!("../fonts/SeoulNamsanvert.otf"),
    include_bytes!("../fonts/Roboto-VariableFont_wdth,wght.ttf"),
    include_bytes!("../fonts/Roboto-Italic-VariableFont_wdth,wght.ttf"),
    include_bytes!("../fonts/NotoSansHebrew-VariableFont_wdth,wght.ttf"),
];

enum HorizontalAlignment {
    Normal,
    Reverse,
    Center,
}

impl Default for HorizontalAlignment {
    fn default() -> Self {
        HorizontalAlignment::Normal
    }
}

enum VerticalAlignment {
    Normal,
    Reverse,
    Center,
}

impl Default for VerticalAlignment {
    fn default() -> Self {
        VerticalAlignment::Normal
    }
}

struct Input {
    text: String,
    paragraphs_fonts: Vec<FontId>,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    fallback_font: FontId,
}

impl<'a> AppState<'a> {
    fn new() -> AppState<'a> {
        let mut fonts = HashMap::<FontId, Font<'a>>::new();

        fonts.insert(
            GLOBAL_FALLBACK_FONT.into(),
            Font {
                raw_data: FONT_DATA[0],
                face: hb::Face::from_slice(FONT_DATA[0], 0).unwrap(),
            },
        );
        fonts.insert(
            "seoul".into(),
            Font {
                raw_data: FONT_DATA[1],
                face: hb::Face::from_slice(FONT_DATA[1], 0).unwrap(),
            },
        );

        let mut roboto = Font {
            raw_data: FONT_DATA[2],
            face: hb::Face::from_slice(FONT_DATA[2], 0).unwrap(),
        };
        roboto
            .face
            .set_variation(hb::ttf_parser::Tag::from_bytes(b"wght"), 400.0);
        fonts.insert("roboto".into(), roboto);

        let mut roboto_italic = Font {
            raw_data: FONT_DATA[3],
            face: hb::Face::from_slice(FONT_DATA[3], 0).unwrap(),
        };
        roboto_italic
            .face
            .set_variation(hb::ttf_parser::Tag::from_bytes(b"wght"), 600.0);
        fonts.insert("roboto-italic".into(), roboto_italic);

        let mut noto = Font {
            raw_data: FONT_DATA[4],
            face: hb::Face::from_slice(FONT_DATA[4], 0).unwrap(),
        };
        noto.face
            .set_variation(hb::ttf_parser::Tag::from_bytes(b"wght"), 400.0);
        fonts.insert("noto".into(), noto);

        let inputs = vec![
            Input {
                text: "아무도 자의적인 체포, 구금 또는 추방을 당하지 않아야 합니다. 모든 사람은 자신의 권리와 의무, 그리고 자신에게 제기된 형사 혐의를 결정함에 있어 독립적이고 공정한 재판소에 의해 평등하게 공정하고 공개적인 심리를 받을 권리를 갖습니다. 아무도 자신의 사생활, 가족, 가정 또는 서신에 대한 자의적인 간섭이나 명예와 평판에 대한 공격을 받아서는 안 됩니다. 모든 사람은 그러한 간섭이나 공격으로부터 법의 보호를 받을 권리를 갖습니다.".into(),
                paragraphs_fonts: vec!["seoul".into()],
                fallback_font: "seoul".into(),
                horizontal_alignment: HorizontalAlignment::Normal,
                vertical_alignment: VerticalAlignment::Normal,
            },
            Input {
                text: "איש לא יהיה נתון למעצר, מעצר שרירותי או גירוש. לכל אדם הזכות לשוויון מלא למשפט הוגן ופומבי בפני בית דין עצמאי ובלתי משוחד, לצורך הכרעה בזכויותיו וחובותיו ובכל אישום פלילי המופנה נגדו. איש לא יהיה נתון להתערבות שרירותית בפרטיותו, במשפחתו, בביתו או בהתכתבויותיו, ולא לפגיעות בכבודו או בשמו הטוב. לכל אדם הזכות להגנת החוק מפני התערבויות או פגיעות כאלה.".into(),
                paragraphs_fonts: vec!["noto".into()],
                fallback_font: "noto".into(),
                horizontal_alignment: HorizontalAlignment::Normal,
                vertical_alignment: VerticalAlignment::Normal,
            },
            Input {
                text: "Nul ne sera soumis à une arrestation, une détention ou un exil arbitraires.\n\nToute personne a droit, en pleine égalité, à ce que sa cause soit entendue équitablement et publiquement par un tribunal indépendant et impartial, qui décidera de ses droits et obligations ainsi que du bien-fondé de toute accusation en matière pénale portée contre elle. Nul ne sera l'objet d'immixtions arbitraires dans sa vie privée, sa famille, son domicile ou sa correspondance, ni d'atteintes à son honneur et à sa réputation. Toute personne a droit à la protection de la loi contre de telles immixtions ou de telles atteintes.\nFin.\n\n".into(),
                paragraphs_fonts: vec!["pt".into(), "pt".into(), "pt".into(), "pt".into(), "pt".into(), "pt".into()],
                fallback_font: "pt".into(),
                horizontal_alignment: HorizontalAlignment::Normal,
                vertical_alignment: VerticalAlignment::Normal,
            },
            Input {
                text: "Nul ne sera soumis à une arrestation, une détention ou un exil arbitraires. \n איש לא יהיה נתון להתערבות שרירותית בפרטיותו, במשפחתו, בביתו או בהתכתבויותיו, ולא לפגיעות בכבודו או בשמו הטוב\nToute personne a droit à la protection de la loi contre de telles immixtions ou de telles atteintes.".into(),
                paragraphs_fonts: vec!["roboto-italic".into(), "noto".into(), "roboto".into()],
                fallback_font: "roboto".into(),
                horizontal_alignment: HorizontalAlignment::Normal,
                vertical_alignment: VerticalAlignment::Normal,
            }
        ];

        AppState::<'a> {
            fonts,
            inputs,
            last_input: 0,
            last_text_size: 16,
            already_performed_layout: false,
        }
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
            let display_str: String = String::from(if i == (bidi_info.paragraphs.len() - 1) {
                let initial_guess = &self.inputs[input].text[line.start..line.end];
                if initial_guess.ends_with("\n") {
                    &self.inputs[input].text[line.start..line.end - 1]
                } else {
                    initial_guess
                }
            } else {
                &self.inputs[input].text[line.start..line.end - 1]
            });
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
        const PAD: f64 = 12.0;
        let line_height = 1.25 * (input_transform.size as f64);
        let max_line_length = (input_transform.w as f64 - 2.0 * PAD).max(0.0);
        let mut result = vec![];

        let mut shaped_paragraphs = Vec::<ParagraphInfo>::with_capacity(paragraphs.len());
        let mut total_number_of_lines = 0;

        for (text, font, is_rtl) in paragraphs.iter() {
            let shaped_fragments =
                self.shape_static_text(text, &font.face, input_transform, *is_rtl);
            let paragraph = ParagraphInfo::new(shaped_fragments, max_line_length, *is_rtl);
            total_number_of_lines += paragraph.lines.len();
            shaped_paragraphs.push(paragraph);
        }

        let mut current_height = (input_transform.y as f64) + PAD + line_height;

        for paragraph in shaped_paragraphs.iter_mut() {
            let is_rtl = paragraph.is_rtl;

            for line in paragraph.lines.iter() {
                let mut baseline = if is_rtl {
                    DVec2::new(
                        ((input_transform.x + input_transform.w) as f64) - PAD,
                        current_height,
                    )
                } else {
                    DVec2::new((input_transform.x as f64) + PAD, current_height)
                };

                let start = line.first_fragment_index;
                let end = if line.has_next_line {
                    line.last_fragment_index
                } else {
                    paragraph.shaped_fragments.len()
                };

                for fragment in paragraph.shaped_fragments[start..end].iter_mut() {
                    let new_baseline_x = if is_rtl {
                        baseline.x - fragment.length
                    } else {
                        baseline.x
                    };

                    let offset = DVec2::new(new_baseline_x, baseline.y);
                    for glyph in fragment.glyphs.iter_mut() {
                        glyph.translate(offset);
                        result.push(glyph.svg_path_string.clone());
                    }

                    baseline.x = if is_rtl {
                        new_baseline_x
                    } else {
                        new_baseline_x + fragment.length
                    };
                }

                current_height += line_height;
            }
        }

        result
    }

    fn shape_static_text(
        &self,
        text: &str,
        face: &hb::Face,
        input_transform: &InputTransform,
        is_rtl: bool,
    ) -> Vec<ShapedFragment> {
        let mut result = vec![];
        use icu::segmenter::LineSegmenter;
        let segmenter = LineSegmenter::new_auto();

        let mut prev_segment_index = 0;
        for segment in segmenter.segment_str(text) {
            let pre_context = &text[0..prev_segment_index];
            let current_text = &text[prev_segment_index..segment];
            let post_context = &text[segment..];

            let mut buffer = hb::UnicodeBuffer::new();
            buffer.set_pre_context(pre_context);
            buffer.push_str(current_text);
            buffer.set_post_context(post_context);
            buffer.guess_segment_properties();
            if is_rtl {
                buffer.set_direction(hb::Direction::RightToLeft);
            } else {
                buffer.set_direction(hb::Direction::LeftToRight);
            }
            buffer.set_cluster_level(hb::BufferClusterLevel::MonotoneCharacters);

            let glyph_buffer = hb::shape(face, &[], buffer);
            let shaped_glyphs = Self::perform_shaping(&glyph_buffer, face, input_transform);
            let shaped_fragment = ShapedFragment::new(shaped_glyphs);

            // Don't keep empty segments. They are an often occurence because a line break can always
            // be inserted before the first letter of a paragraph.
            if !current_text.is_empty() {
                result.push(shaped_fragment);
            }
            prev_segment_index = segment;
        }

        result
    }

    fn perform_shaping(
        glyph_buffer: &hb::GlyphBuffer,
        face: &hb::Face,
        input_transform: &InputTransform,
    ) -> Vec<GlyphPath> {
        let mut result = vec![];
        let mut baseline = DVec2::new(0.0, 0.0);

        for (glyph, info) in glyph_buffer
            .glyph_positions()
            .iter()
            .zip(glyph_buffer.glyph_infos().iter())
        {
            let glyph_id = hb::ttf_parser::GlyphId(info.glyph_id.try_into().unwrap());
            let font_transform = Self::from_font_space_to_screen_space(&face, input_transform.size);

            let (advance_x, advance_y, offset_x, offset_y) = (
                glyph.x_advance,
                glyph.y_advance,
                glyph.x_offset,
                glyph.y_offset,
            );
            let advance = DVec2::new(advance_x as f64, advance_y as f64);
            let advance = font_transform.transform_point2(advance);

            let offset = DVec2::new(offset_x as f64, offset_y as f64);
            let glyph_transform = DAffine2::from_translation(baseline)
                * font_transform
                * DAffine2::from_translation(offset);
            let mut glyph_path = GlyphPath {
                svg_path_string: "".into(),
                transform: glyph_transform,
                cmds: vec![],
                advance_x: advance.x,
            };
            face.outline_glyph(glyph_id, &mut glyph_path);

            result.push(glyph_path);
            baseline += advance;
        }

        result
    }

    fn from_font_space_to_screen_space(face: &hb::Face, text_size: usize) -> DAffine2 {
        let units_per_em = face.units_per_em();
        let (ppem, upem) = (text_size as f64, units_per_em as f64);
        // `ppem` gives us the mapping between font units and screen pixels.
        // ppem stands for pixels per em.
        let to_px = ppem / upem;

        DAffine2::from_scale(DVec2::new(to_px, -to_px))
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

struct ShapedFragment {
    glyphs: Vec<GlyphPath>,
    length: f64,
}

impl ShapedFragment {
    fn new(glyphs: Vec<GlyphPath>) -> Self {
        let mut length = 0.0;

        for glyph in glyphs.iter() {
            length += glyph.advance_x;
        }

        Self { glyphs, length }
    }
}

#[derive(Debug, Clone)]
enum PathCmd {
    M(DVec2),
    L(DVec2),
    Q(DVec2, DVec2),
    C(DVec2, DVec2, DVec2),
    Z,
}

#[derive(Debug, Clone)]
struct GlyphPath {
    svg_path_string: String,
    transform: DAffine2,
    cmds: Vec<PathCmd>,
    advance_x: f64,
}

impl hb::ttf_parser::OutlineBuilder for GlyphPath {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = DVec2::new(x as f64, y as f64);
        let to = self.transform.transform_point2(to);
        self.svg_path_string += &format!("M{} {} ", to.x, to.y);
        self.cmds.push(PathCmd::M(to));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = DVec2::new(x as f64, y as f64);
        let to = self.transform.transform_point2(to);
        self.svg_path_string += &format!("L{} {} ", to.x, to.y);
        self.cmds.push(PathCmd::L(to));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = DVec2::new(x1 as f64, y1 as f64);
        let p2 = DVec2::new(x as f64, y as f64);

        let p1 = self.transform.transform_point2(p1);
        let p2 = self.transform.transform_point2(p2);

        self.svg_path_string += &format!("Q{} {},{} {} ", p1.x, p1.y, p2.x, p2.y);
        self.cmds.push(PathCmd::Q(p1, p2));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p1 = DVec2::new(x1 as f64, y1 as f64);
        let p1 = self.transform.transform_point2(p1);
        let p2 = DVec2::new(x2 as f64, y2 as f64);
        let p2 = self.transform.transform_point2(p2);
        let p3 = DVec2::new(x as f64, y as f64);
        let p3 = self.transform.transform_point2(p3);

        self.svg_path_string += &format!("C{} {},{} {},{} {} ", p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
        self.cmds.push(PathCmd::C(p1, p2, p3));
    }

    fn close(&mut self) {
        self.svg_path_string += "Z ";
        self.cmds.push(PathCmd::Z);
    }
}

impl GlyphPath {
    fn translate(&mut self, offset: DVec2) {
        self.svg_path_string.clear();
        self.cmds.iter().for_each(|cmd| match cmd {
            PathCmd::M(to) => {
                let to = *to + offset;
                self.svg_path_string += &format!("M{} {} ", to.x, to.y);
            }
            PathCmd::L(to) => {
                let to = *to + offset;
                self.svg_path_string += &format!("L{} {} ", to.x, to.y);
            }
            PathCmd::Q(p1, p2) => {
                let p1 = *p1 + offset;
                let p2 = *p2 + offset;
                self.svg_path_string += &format!("Q{} {},{} {} ", p1.x, p1.y, p2.x, p2.y);
            }
            PathCmd::C(p1, p2, p3) => {
                let p1 = *p1 + offset;
                let p2 = *p2 + offset;
                let p3 = *p3 + offset;
                self.svg_path_string +=
                    &format!("C{} {},{} {},{} {} ", p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
            }
            PathCmd::Z => {
                self.svg_path_string += "Z ";
            }
        });
    }
}

struct LineInfo {
    first_fragment_index: usize,
    last_fragment_index: usize,
    line_length: f64,
    has_next_line: bool,
}

struct ParagraphInfo {
    shaped_fragments: Vec<ShapedFragment>,
    lines: Vec<LineInfo>,
    is_rtl: bool,
}

impl ParagraphInfo {
    fn new(shaped_fragments: Vec<ShapedFragment>, max_line_length: f64, is_rtl: bool) -> Self {
        let mut lines = vec![];

        lines.push(LineInfo {
            first_fragment_index: 0,
            last_fragment_index: 0,
            line_length: 0.0,
            has_next_line: false,
        });

        let mut current_line_length = 0.0;

        for (i, fragment) in shaped_fragments.iter().enumerate() {
            current_line_length += fragment.length;

            if current_line_length > max_line_length {
                current_line_length = fragment.length;

                if i > 0 {
                    lines.last_mut().unwrap().last_fragment_index = i;
                    lines.last_mut().unwrap().has_next_line = true;
                    lines.push(LineInfo {
                        first_fragment_index: i,
                        last_fragment_index: i,
                        line_length: fragment.length,
                        has_next_line: false,
                    });
                } else {
                    lines.last_mut().unwrap().line_length = current_line_length;
                }
            } else {
                lines.last_mut().unwrap().line_length += fragment.length;
            }
        }

        Self {
            shaped_fragments,
            lines,
            is_rtl,
        }
    }
}
