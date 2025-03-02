use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Once;

use rustybuzz as hb; // alias for harfbuzz
use wasm_bindgen::prelude::*;

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
    fallback_font: FontId,
}

impl<'a> AppState<'a> {
    fn new() -> AppState<'a> {
        let mut fonts = HashMap::<FontId, Font<'a>>::new();

        fonts.insert(
            "pt".into(),
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
                fallback_font: "seoul".into(),
            },
            Input {
                text: "איש לא יהיה נתון למעצר, מעצר שרירותי או גירוש. לכל אדם הזכות לשוויון מלא למשפט הוגן ופומבי בפני בית דין עצמאי ובלתי משוחד, לצורך הכרעה בזכויותיו וחובותיו ובכל אישום פלילי המופנה נגדו. איש לא יהיה נתון להתערבות שרירותית בפרטיותו, במשפחתו, בביתו או בהתכתבויותיו, ולא לפגיעות בכבודו או בשמו הטוב. לכל אדם הזכות להגנת החוק מפני התערבויות או פגיעות כאלה.".into(),
                spans: vec![],
                fallback_font: "noto".into(),
            },
            Input {
                text: "Nul ne sera soumis à une arrestation, une détention ou un exil arbitraires. Toute personne a droit, en pleine égalité, à ce que sa cause soit entendue équitablement et publiquement par un tribunal indépendant et impartial, qui décidera de ses droits et obligations ainsi que du bien-fondé de toute accusation en matière pénale portée contre elle. Nul ne sera l'objet d'immixtions arbitraires dans sa vie privée, sa famille, son domicile ou sa correspondance, ni d'atteintes à son honneur et à sa réputation. Toute personne a droit à la protection de la loi contre de telles immixtions ou de telles atteintes.".into(),
                spans: vec![],
                fallback_font: "pt".into(),
            },
            Input {
                text: "Nul ne sera soumis à une arrestation, une détention ou un exil arbitraires. \n איש לא יהיה נתון להתערבות שרירותית בפרטיותו, במשפחתו, בביתו או בהתכתבויותיו, ולא לפגיעות בכבודו או בשמו הטוב\n Toute personne a droit à la protection de la loi contre de telles immixtions ou de telles atteintes.".into(),
                spans: vec![],
                fallback_font: "roboto".into(),
            }
        ];

        AppState::<'a> { fonts, inputs }
    }

    fn resolve_input(&self, input_transform: &InputTransform, input: usize) -> Vec<String> {
        let InputTransform {
            x,
            y,
            w,
            h,
            size: _,
        } = input_transform;

        use icu::properties::bidi::BidiClassAdapter;
        use icu::properties::maps;
        use unicode_bidi::BidiInfo;

        let adapter = BidiClassAdapter::new(maps::bidi_class());
        let bidi_info =
            BidiInfo::new_with_data_source(&adapter, self.inputs[input].text.as_ref(), None);
        let paragraph = &bidi_info.paragraphs[0];
        let line = paragraph.range.clone();
        let display_str = bidi_info.reorder_line(paragraph, line);

        let msg = format!(
            "Num paragraphs: {}, embedding level of first paragraph: {}, display_str: {}",
            bidi_info.paragraphs.len(),
            paragraph.level.number(),
            display_str,
        );
        web_sys::console::log_1(&msg.into());

        vec![format!("M{} {} h{} v{} h{} Z", x, y, w, h, -w)]
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
