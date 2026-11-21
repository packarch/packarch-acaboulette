//! Acaboulette — un gros matou roux traverse Bordeaux au pas de course et
//! mange ou ecrabouille tout ce qui passe : pigeons de la Bourse, rats de la
//! Garonne, chiens, barrages de CRS... Seuls le mobilier urbain, les mines, les voitures, les motos et le tram B font mal.
//!
//! Commandes :
//!   Espace / clic / fleche haut   : sauter (double saut ; maintenir = plus haut)
//!                                   3e saut : deltaplane, chaque clic redresse le nez
//!   Fleche bas / S / clic droit   : plat ventre (en l'air) : BADABOUM
//!   Dans le char d'assaut           : clic gauche = obus, clic droit = missiles
//!   G                             : activer / desactiver le gore
//!   Echap                         : quitter

// Pas de console qui s'ouvre derriere le jeu sous Windows (en release).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Constantes de jeu (unites : metres, secondes)
// ---------------------------------------------------------------------------

const DT: f32 = 1.0 / 120.0;
const GRAV: f32 = 34.0;
const JUMP_V: f32 = 13.0;
const HOLD_FACTOR: f32 = 0.62; // gravite reduite tant qu'on maintient le saut
const SLAM_V: f32 = -28.0;
const TANK_TIME: f32 = 14.0; // duree d'un tour en char d'assaut
const TANK_SPEED: f32 = 6.5; // le char ne roule pas trop vite
const TANK_LEN: f32 = 4.6;
const TANK_H: f32 = 2.1;
const TANK_SEAT: f32 = 1.8; // position du chat depuis l'arriere du char
const TANK_MIN_X: f32 = 250.0; // pas de char avant cette distance
const TANK_EVERY: f32 = 400.0; // distance minimale entre deux chars
const TANK_CHANCE: f32 = 0.06;
const SHELL_SPEED: f32 = 42.0;
const SHELL_CD: f32 = 0.7;
const MISSILE_CD: f32 = 1.4;
const MISSILE_N: usize = 5;
const GLIDE_MAX: f32 = 4.0; // duree du vol en deltaplane (le 3e saut)
const GLIDE_GRAV: f32 = 9.0; // gravite sous l'aile
const GLIDE_FLARE_COST: f32 = 0.3; // chaque coup de nez raccourcit le vol
const START_SPEED: f32 = 7.5;
const MAX_SPEED: f32 = 15.0;
const SPEED_GAIN: f32 = 0.07; // m/s gagnes chaque seconde
const LIVES: u32 = 3;
const MAX_FAT: f32 = 12.0;
const ZONE_LEN: f32 = 260.0;
const TRAM_LEN: f32 = 18.0;
const TRAM_TOP: f32 = 2.9;
const TRAM_SPEED: f32 = 5.0;
const GAVE_TIME: f32 = 7.0;
const KEBAB_TIME: f32 = 12.0; // super vitesse + indestructible
const KEBAB_SPEED: f32 = 1.9;
const CLOSEUP_TIME: f32 = 2.6; // pause gros plan quand il avale le kebab
const LITTER_TIME: f32 = 3.2; // pause litiere quand il devient trop gros

#[derive(Clone, Copy, PartialEq)]
enum Closeup {
    Kebab,
    Litter,
}
const SPAWN_AHEAD: f32 = 32.0;

const SAVE_FILE: &str = "record.txt";

// Couleurs
const GINGER: Color = Color::new(0.95, 0.55, 0.18, 1.0);
const GINGER_DARK: Color = Color::new(0.76, 0.35, 0.08, 1.0);
const GINGER_SHADE: Color = Color::new(0.84, 0.45, 0.13, 1.0);
const CREAM: Color = Color::new(1.0, 0.93, 0.8, 1.0);
const NOSE: Color = Color::new(0.95, 0.5, 0.56, 1.0);
const INK: Color = Color::new(0.1, 0.08, 0.08, 1.0);
const STONE: Color = Color::new(0.89, 0.81, 0.63, 1.0); // pierre blonde de Bordeaux
const STONE_SHADE: Color = Color::new(0.76, 0.67, 0.5, 1.0);
const STONE_DARK: Color = Color::new(0.6, 0.52, 0.38, 1.0);
const SLATE: Color = Color::new(0.33, 0.37, 0.45, 1.0);
const GARONNE: Color = Color::new(0.58, 0.47, 0.32, 1.0); // couleur café au lait
const BRICK: Color = Color::new(0.78, 0.5, 0.42, 1.0);
const BORDEAUX: Color = Color::new(0.45, 0.05, 0.16, 1.0);
const TANK_JOINT_TIP: Vec2 = Vec2::new(3.2, 2.2); // bout du joint, depuis l'arriere du char
const TANK_GREEN: Color = Color::new(0.37, 0.44, 0.22, 1.0);
const TANK_DARK: Color = Color::new(0.24, 0.3, 0.14, 1.0);
const MED_SEA: Color = Color::new(0.1, 0.45, 0.72, 1.0); // la Mediterranee
const TILE: Color = Color::new(0.82, 0.42, 0.27, 1.0); // tuiles romaines
const LIMESTONE: Color = Color::new(0.9, 0.88, 0.82, 1.0); // calcaire des calanques
const LAVENDER: Color = Color::new(0.56, 0.44, 0.78, 1.0);
const SAND: Color = Color::new(0.93, 0.78, 0.5, 1.0);
const BONE: Color = Color::new(0.96, 0.93, 0.84, 1.0);

fn blood(shade: f32) -> Color {
    Color::new(0.55 + 0.3 * shade, 0.0, 0.02 + 0.04 * shade, 1.0)
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn darker(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

/// Bruit deterministe dans [0, 1) : sert a dessiner la ville toujours pareil.
fn hash(i: i32, salt: u32) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// Deltaplane : vitesse verticale donnee par un coup de nez, apres `t`
/// secondes de vol. Elle s'epuise : la gravite finit toujours par gagner.
fn glide_lift(t: f32) -> f32 {
    5.5 * (1.0 - t / GLIDE_MAX).max(0.0)
}

/// Deltaplane : vitesse de chute maximale, qui augmente au fil du vol.
fn glide_sink(t: f32) -> f32 {
    1.4 + 2.6 * (t / GLIDE_MAX).clamp(0.0, 1.0)
}

fn pick<'a>(list: &[&'a str]) -> &'a str {
    list[gen_range(0, list.len())]
}

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

#[cfg(not(windows))]
fn data_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("packarch-acaboulette"))
}

// Sous Windows : %APPDATA%\packarch-acaboulette
#[cfg(windows)]
fn data_dir() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join("AppData").join("Roaming"))
        })?;
    Some(base.join("packarch-acaboulette"))
}

fn load_record() -> u32 {
    data_dir()
        .and_then(|d| std::fs::read_to_string(d.join(SAVE_FILE)).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_record(value: u32) {
    if let Some(d) = data_dir() {
        let _ = std::fs::create_dir_all(&d);
        let _ = std::fs::write(d.join(SAVE_FILE), value.to_string());
    }
}

// ---------------------------------------------------------------------------
// Les quartiers : Bordeaux, Marseille, Forcalquier, l'Algerie (en boucle)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Zone {
    Chartrons,
    Bourse,
    PontDePierre,
    GrosseCloche,
    SainteCatherine,
    CiteDuVin,
    Chaban,
    // Marseille : on y arrive au petit matin, apres la premiere nuit
    VieuxPort,
    Canebiere,
    Panier,
    BonneMere,
    Velodrome,
    Corniche,
    // Forcalquier, en Haute-Provence
    Bourguet,
    Citadelle,
    Lavande,
    Oliviers,
    Mourres,
    // L'Algerie
    GrandePoste,
    Casbah,
    Maqam,
    NotreDameAfrique,
    Sahara,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum City {
    Bordeaux,
    Marseille,
    Forcalquier,
    Algerie,
}

/// Les villes de depart, dans l'ordre du voyage.
const CITIES: [City; 4] = [City::Bordeaux, City::Marseille, City::Forcalquier, City::Algerie];

impl City {
    fn name(self) -> &'static str {
        match self {
            City::Bordeaux => "Bordeaux",
            City::Marseille => "Marseille",
            City::Forcalquier => "Forcalquier",
            City::Algerie => "Algérie",
        }
    }
    /// Rang de son premier quartier dans la boucle.
    fn first_zone(self) -> usize {
        ZONES.iter().position(|z| z.city() == self).unwrap_or(0)
    }
}

/// Ville de depart choisie sur l'ecran titre : decale toute la boucle des
/// quartiers (la partie commence toujours a x = 0).
static START_CITY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn start_city() -> usize {
    START_CITY.load(std::sync::atomic::Ordering::Relaxed).min(CITIES.len() - 1)
}

fn set_start_city(k: usize) {
    START_CITY.store(k % CITIES.len(), std::sync::atomic::Ordering::Relaxed);
}

const ZONES: [Zone; 23] = [
    Zone::Chartrons,
    Zone::Bourse,
    Zone::PontDePierre,
    Zone::GrosseCloche,
    Zone::SainteCatherine,
    Zone::CiteDuVin,
    Zone::Chaban,
    Zone::VieuxPort,
    Zone::Canebiere,
    Zone::Panier,
    Zone::BonneMere,
    Zone::Velodrome,
    Zone::Corniche,
    Zone::Bourguet,
    Zone::Citadelle,
    Zone::Lavande,
    Zone::Oliviers,
    Zone::Mourres,
    Zone::GrandePoste,
    Zone::Casbah,
    Zone::Maqam,
    Zone::NotreDameAfrique,
    Zone::Sahara,
];

fn zone_index(x: f32) -> i32 {
    (x / ZONE_LEN).floor() as i32
}

/// Quartier de rang `zi` (rang compte depuis le depart de la partie).
fn zone_of(zi: i32) -> Zone {
    let shift = CITIES[start_city()].first_zone() as i32;
    ZONES[(zi + shift).rem_euclid(ZONES.len() as i32) as usize]
}

fn zone_at(x: f32) -> Zone {
    zone_of(zone_index(x))
}

impl Zone {
    fn city(self) -> City {
        match self {
            Zone::Chartrons | Zone::Bourse | Zone::PontDePierre | Zone::GrosseCloche | Zone::SainteCatherine | Zone::CiteDuVin | Zone::Chaban => City::Bordeaux,
            Zone::VieuxPort | Zone::Canebiere | Zone::Panier | Zone::BonneMere | Zone::Velodrome | Zone::Corniche => City::Marseille,
            Zone::Bourguet | Zone::Citadelle | Zone::Lavande | Zone::Oliviers | Zone::Mourres => City::Forcalquier,
            Zone::GrandePoste | Zone::Casbah | Zone::Maqam | Zone::NotreDameAfrique | Zone::Sahara => City::Algerie,
        }
    }
    /// Couleur de la bande derriere les quais : fleuve, mer, champ ou sable.
    fn band(self) -> Color {
        match self {
            Zone::Lavande => LAVENDER,
            Zone::Oliviers => Color::new(0.74, 0.7, 0.46, 1.0), // herbe seche
            Zone::Mourres => Color::new(0.66, 0.66, 0.42, 1.0), // garrigue
            Zone::Sahara => SAND,
            _ => match self.city() {
                City::Bordeaux => GARONNE,
                City::Marseille => MED_SEA,
                City::Forcalquier => LAVENDER,
                City::Algerie => Color::new(0.1, 0.5, 0.7, 1.0), // la baie d'Alger
            },
        }
    }
    /// Grand ciel bleu sans nuages (partout sauf a Bordeaux).
    fn sunny(self) -> f32 {
        if self.city() == City::Bordeaux { 0.0 } else { 1.0 }
    }
    fn name(self) -> &'static str {
        match self {
            Zone::Chartrons => "Quai des Chartrons",
            Zone::Bourse => "Place de la Bourse et Miroir d'eau",
            Zone::PontDePierre => "Pont de pierre",
            Zone::GrosseCloche => "La Grosse Cloche",
            Zone::SainteCatherine => "Rue Sainte-Catherine",
            Zone::CiteDuVin => "La Cité du Vin",
            Zone::Chaban => "Pont Chaban-Delmas",
            Zone::VieuxPort => "Marseille : le Vieux-Port",
            Zone::Canebiere => "La Canebière",
            Zone::Panier => "Le Panier",
            Zone::BonneMere => "Notre-Dame de la Garde",
            Zone::Velodrome => "Stade Vélodrome",
            Zone::Corniche => "La Corniche Kennedy",
            Zone::Bourguet => "Forcalquier : place du Bourguet",
            Zone::Citadelle => "La Citadelle",
            Zone::Lavande => "Les champs de lavande",
            Zone::Oliviers => "L'oliveraie",
            Zone::Mourres => "Les Mourres",
            Zone::GrandePoste => "Alger : la Grande Poste",
            Zone::Casbah => "La Casbah",
            Zone::Maqam => "Maqam Echahid",
            Zone::NotreDameAfrique => "Notre-Dame d'Afrique",
            Zone::Sahara => "Le Sahara",
        }
    }
    fn subtitle(self) -> &'static str {
        match self {
            Zone::Chartrons => "Les pigeons des Chartrons sont tarpin gras",
            Zone::Bourse => "Ça glisse sur le Miroir d'eau !",
            Zone::PontDePierre => "La Garonne, couleur café au lait",
            Zone::GrosseCloche => "La Grosse Cloche sonne pour toi",
            Zone::SainteCatherine => "Il pleut. Normal, c'est Bordeaux.",
            Zone::CiteDuVin => "Un petit canon de Saint-Émilion ?",
            Zone::Chaban => "Le tablier monte, les CRS descendent",
            Zone::VieuxPort => "Ici, « tarpin », ça se dit pour de vrai. Peuchère !",
            Zone::Canebiere => "Elle descend jusqu'au port, et le matou avec",
            Zone::Panier => "Ça sent le savon et le linge qui sèche",
            Zone::BonneMere => "La Bonne Mère veille sur le gros matou",
            Zone::Velodrome => "Allez l'OM ! Les Girondins ? Connais pas.",
            Zone::Corniche => "Au large, le château d'If. Tarpin beau.",
            Zone::Bourguet => "Lundi, jour de marché : ça sent l'olive et le pastis",
            Zone::Citadelle => "Le carillon sonne, les pigeons s'envolent",
            Zone::Lavande => "Ça sent tarpin bon. Atchoum !",
            Zone::Oliviers => "Des oliviers centenaires. Ici, l'huile, c'est sacré.",
            Zone::Mourres => "Des champignons de pierre. Ça ne se mange pas.",
            Zone::GrandePoste => "Alger la Blanche, face à la baie",
            Zone::Casbah => "Des ruelles, des escaliers, et des chats partout",
            Zone::Maqam => "Le Mémorial du Martyr domine la baie",
            Zone::NotreDameAfrique => "Elle veille sur Alger, comme la Bonne Mère sur Marseille",
            Zone::Sahara => "Du sable, du sable... la plus grande litière du monde",
        }
    }
    /// Pas de facades : on voit la Garonne.
    fn river(self) -> bool {
        matches!(
            self,
            Zone::PontDePierre | Zone::CiteDuVin | Zone::Chaban | Zone::VieuxPort | Zone::Corniche | Zone::Lavande | Zone::Oliviers | Zone::Mourres | Zone::Maqam | Zone::Sahara
        )
    }
    /// Demi-largeur du monument (coordonnees du plan lointain).
    fn landmark_half(self) -> f32 {
        match self {
            Zone::Chartrons => 7.0,
            Zone::Bourse => 24.0,
            Zone::PontDePierre => 0.0,
            Zone::GrosseCloche => 8.0,
            Zone::SainteCatherine => 13.0,
            Zone::CiteDuVin => 0.0,
            Zone::Chaban => 0.0,
            Zone::VieuxPort => 0.0,
            Zone::Canebiere => 7.0,
            Zone::Panier => 11.0,
            Zone::BonneMere => 15.0,
            Zone::Velodrome => 18.0,
            Zone::Corniche => 0.0,
            Zone::Bourguet => 9.0,
            Zone::Citadelle => 14.0,
            Zone::Lavande => 0.0,
            Zone::Oliviers => 0.0,
            Zone::Mourres => 0.0,
            Zone::GrandePoste => 11.0,
            Zone::Casbah => 9.0,
            Zone::Maqam => 0.0,
            Zone::NotreDameAfrique => 14.0,
            Zone::Sahara => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Camera / transformation monde -> ecran
// ---------------------------------------------------------------------------

struct View {
    cam_x: f32,
    cam_y: f32,
    s: f32, // pixels par metre
    ground_sy: f32,
    anchor_x: f32,
    shake: Vec2,
}

impl View {
    fn p(&self, w: Vec2) -> Vec2 {
        vec2(
            self.anchor_x + (w.x - self.cam_x) * self.s + self.shake.x,
            self.ground_sy - (w.y - self.cam_y) * self.s + self.shake.y,
        )
    }
    fn l(&self, m: f32) -> f32 {
        (m * self.s).max(1.0)
    }
    /// Plan de decor en parallaxe : `f` = vitesse relative, `k` = echelle.
    fn layer(&self, f: f32, k: f32, base_sy: f32) -> View {
        View {
            cam_x: self.cam_x * f,
            cam_y: 0.0,
            s: self.s * k,
            ground_sy: base_sy,
            anchor_x: self.anchor_x,
            shake: self.shake * k,
        }
    }
    /// Abscisses visibles (dans les coordonnees de ce plan).
    fn x_range(&self) -> (f32, f32) {
        let l = self.cam_x - self.anchor_x / self.s;
        let r = self.cam_x + (screen_width() - self.anchor_x) / self.s;
        (l, r)
    }
}

fn ellipse(v: &View, c: Vec2, rx: f32, ry: f32, ang: f32, col: Color) {
    const N: usize = 22;
    let rot = Vec2::from_angle(ang);
    let center = v.p(c);
    let pt = |i: usize| {
        let t = i as f32 / N as f32 * TAU;
        v.p(c + rot.rotate(vec2(rx * t.cos(), ry * t.sin())))
    };
    let mut prev = pt(0);
    for i in 1..=N {
        let cur = pt(i);
        draw_triangle(center, prev, cur, col);
        prev = cur;
    }
}

fn circ(v: &View, c: Vec2, r: f32, col: Color) {
    let p = v.p(c);
    draw_circle(p.x, p.y, v.l(r), col);
}

fn seg(v: &View, a: Vec2, b: Vec2, th: f32, col: Color) {
    let (pa, pb) = (v.p(a), v.p(b));
    let t = v.l(th);
    draw_line(pa.x, pa.y, pb.x, pb.y, t, col);
    draw_circle(pa.x, pa.y, t * 0.5, col);
    draw_circle(pb.x, pb.y, t * 0.5, col);
}

fn line(v: &View, a: Vec2, b: Vec2, th: f32, col: Color) {
    let (pa, pb) = (v.p(a), v.p(b));
    draw_line(pa.x, pa.y, pb.x, pb.y, (th * v.s).max(1.0), col);
}

fn tri(v: &View, a: Vec2, b: Vec2, c: Vec2, col: Color) {
    draw_triangle(v.p(a), v.p(b), v.p(c), col);
}

/// Rectangle monde : (x0, y0) en bas a gauche, (x1, y1) en haut a droite.
fn rect(v: &View, x0: f32, y0: f32, x1: f32, y1: f32, col: Color) {
    let a = v.p(vec2(x0, y1));
    let b = v.p(vec2(x1, y0));
    draw_rectangle(a.x, a.y, b.x - a.x, b.y - a.y, col);
}

/// Demi-ellipse (arche, dome) posee sur y = cy.
fn half_ellipse(v: &View, cx: f32, cy: f32, rx: f32, ry: f32, col: Color) {
    const N: usize = 16;
    let base = v.p(vec2(cx, cy));
    let pt = |i: usize| {
        let t = i as f32 / N as f32 * PI;
        v.p(vec2(cx + rx * t.cos(), cy + ry * t.sin()))
    };
    let mut prev = pt(0);
    for i in 1..=N {
        let cur = pt(i);
        draw_triangle(base, prev, cur, col);
        prev = cur;
    }
}

fn round_rect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    draw_rectangle(x + r, y, w - 2.0 * r, h, c);
    draw_rectangle(x, y + r, r, h - 2.0 * r, c);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, c);
    for (cx, cy) in [(x + r, y + r), (x + w - r, y + r), (x + r, y + h - r), (x + w - r, y + h - r)] {
        draw_circle(cx, cy, r, c);
    }
}

/// Boutons de choix de la ville sur l'ecran titre : (x, y, largeur, hauteur).
fn city_buttons() -> [(f32, f32, f32, f32); 4] {
    let w = screen_width();
    let gap = 14.0;
    let bw = 178.0f32.min((w - 32.0 - 3.0 * gap) / 4.0);
    let x0 = w / 2.0 - (4.0 * bw + 3.0 * gap) / 2.0;
    let y = 414.0f32.min(screen_height() * 0.62);
    std::array::from_fn(|k| (x0 + k as f32 * (bw + gap), y, bw, 46.0))
}

fn city_button_at((mx, my): (f32, f32)) -> Option<usize> {
    city_buttons().iter().position(|&(x, y, w, h)| mx >= x && mx <= x + w && my >= y && my <= y + h)
}

/// Texte facon BD : contour fonce epais, centre sur (x, y).
fn comic_text(txt: &str, x: f32, y: f32, size: u16, fill: Color, outline: Color) {
    let dim = measure_text(txt, None, size, 1.0);
    let tx = x - dim.width / 2.0;
    let ty = y + dim.offset_y / 2.0;
    let o = (size as f32 / 14.0).max(2.0);
    for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (-o, -o), (o, o), (-o, o), (o, -o)] {
        draw_text(txt, tx + dx, ty + dy, size as f32, outline);
    }
    draw_text(txt, tx, ty, size as f32, fill);
}

/// Fond facon BD : rayons qui tournent autour du centre de l'ecran.
fn comic_rays(back: Color, ray: Color, time: f32) {
    let w = screen_width();
    let h = screen_height();
    let center = vec2(w * 0.5, h * 0.56);
    draw_rectangle(0.0, 0.0, w, h, back);
    let n = 24;
    let rr = w.max(h) * 1.2;
    for i in (0..n).step_by(2) {
        let a0 = i as f32 / n as f32 * TAU + time * 0.4;
        let a1 = (i + 1) as f32 / n as f32 * TAU + time * 0.4;
        draw_triangle(center, center + Vec2::from_angle(a0) * rr, center + Vec2::from_angle(a1) * rr, ray);
    }
}

/// Jauge d'un bonus en bas de l'ecran, au milieu ; `slot` 0 tout en bas, 1 au-dessus.
fn bottom_gauge(label: &str, t: f32, warn: bool, back: Color, fill: Color, slot: usize, time: f32) {
    let w = screen_width();
    let h = screen_height();
    let bw = 420.0f32.min(w - 32.0);
    let bx = w / 2.0 - bw / 2.0;
    let by = h - 62.0 - slot as f32 * 54.0;
    round_rect(bx, by, bw, 46.0, 12.0, back);
    round_rect(bx + 10.0, by + 26.0, bw - 20.0, 12.0, 6.0, Color::new(0.2, 0.05, 0.0, 0.9));
    let blink = warn && (time * 12.0).sin() < 0.0;
    let col = if blink { WHITE } else { fill };
    round_rect(bx + 10.0, by + 26.0, ((bw - 20.0) * t.clamp(0.0, 1.0)).max(12.0), 12.0, 6.0, col);
    let dim = measure_text(label, None, 22, 1.0);
    draw_text(label, w / 2.0 - dim.width / 2.0, by + 20.0, 22.0, WHITE);
}

fn centered(txt: &str, y: f32, size: u16, col: Color) {
    let dim = measure_text(txt, None, size, 1.0);
    draw_text(txt, screen_width() / 2.0 - dim.width / 2.0, y, size as f32, col);
}

// ---------------------------------------------------------------------------
// Entites
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Pigeon,
    Rat,
    Dog,
    Crs,
    Gull, // mouette, en altitude : seulement en deltaplane
}

impl Kind {
    fn radius(self) -> f32 {
        match self {
            Kind::Pigeon => 0.22,
            Kind::Rat => 0.2,
            Kind::Dog => 0.3,
            Kind::Crs => 0.45,
            Kind::Gull => 0.3,
        }
    }
    fn points(self) -> f32 {
        match self {
            Kind::Pigeon => 10.0,
            Kind::Rat => 20.0,
            Kind::Dog => 35.0,
            Kind::Crs => 150.0,
            Kind::Gull => 80.0,
        }
    }
    /// Ce que ca fait grossir quand il l'avale.
    fn fat_gain(self) -> f32 {
        match self {
            Kind::Pigeon => 0.07,
            Kind::Rat => 0.11,
            Kind::Dog => 0.22,
            Kind::Crs => 0.0, // on ne mange pas les CRS, on les fait valdinguer
            Kind::Gull => 0.25,
        }
    }
    fn color(self) -> Color {
        match self {
            Kind::Pigeon => Color::new(0.58, 0.6, 0.66, 1.0),
            Kind::Rat => Color::new(0.45, 0.38, 0.33, 1.0),
            Kind::Dog => Color::new(0.72, 0.5, 0.3, 1.0),
            Kind::Crs => Color::new(0.12, 0.16, 0.3, 1.0),
            Kind::Gull => Color::new(0.95, 0.95, 0.97, 1.0),
        }
    }
}

struct Critter {
    kind: Kind,
    pos: Vec2, // bas du corps
    vel: Vec2,
    phase: f32,
    base_y: f32,
    flying: bool,
    flag: bool, // pigeon : a deja decide de fuir ou non
    dead: bool,
}

impl Critter {
    fn center(&self) -> Vec2 {
        self.pos + vec2(0.0, self.kind.radius())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ObsKind {
    Barrier, // barriere de travaux (Bordeaux, ville en travaux depuis toujours)
    Bike,    // velo en libre-service couche en travers
    Bin,     // poubelle
}

struct Obstacle {
    kind: ObsKind,
    x: f32,
    w: f32,
    h: f32,
    knocked: f32, // 0 = debout, > 0 : renverse (angle)
    smashed: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum HazKind {
    Mine, // mine : le chat part en morceaux
}

struct Hazard {
    kind: HazKind,
    x: f32, // centre
    fired: bool,
    t: f32, // temps depuis le declenchement
}

impl HazKind {
    fn half_w(self) -> f32 {
        match self {
            HazKind::Mine => 0.3,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum VehKind {
    Car,
    Moto,
}

/// Voiture ou moto qui arrive en face (vers la gauche).
struct Vehicle {
    kind: VehKind,
    x: f32,
    speed: f32,
    color: Color,
    hit: bool,
    honked: bool,
}

impl Vehicle {
    fn w(&self) -> f32 {
        match self.kind {
            VehKind::Car => 3.6,
            VehKind::Moto => 1.9,
        }
    }
    fn h(&self) -> f32 {
        match self.kind {
            VehKind::Car => 1.45,
            VehKind::Moto => 1.35,
        }
    }
}

/// Char d'assaut a l'arret, qui attend qu'on saute dedans.
struct TankSpot {
    x: f32, // arriere du char
}

/// L'enorme obus du char : il file tout droit.
struct Shell {
    pos: Vec2,
}

/// Missile a tete chercheuse.
struct Missile {
    pos: Vec2,
    vel: Vec2,
    aim: Option<Vec2>,
    age: f32,
}

struct Tram {
    x: f32, // avant du tram (cote gauche, il roule vers la gauche)
    warned: bool,
    rode: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum PickKind {
    Kebab,
}

struct Pickup {
    kind: PickKind,
    pos: Vec2,
    taken: bool,
    phase: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum PK {
    Blood,
    Chunk,
    Feather,
    Fur,
    Star,
    Water,
    Dust,
    Debris,
    Eye,
}

struct Particle {
    kind: PK,
    pos: Vec2,
    vel: Vec2,
    ang: f32,
    spin: f32,
    size: f32,
    col: Color,
    life: f32,
    max_life: f32,
}

/// Un CRS qui valdingue en tournoyant.
struct Flung {
    pos: Vec2,
    vel: Vec2,
    ang: f32,
    spin: f32,
    phase: f32,
}

struct Decal {
    x: f32,
    w: f32,
    shade: f32,
}

struct Splat {
    pos: Vec2, // ecran
    r: f32,
    life: f32,
    max_life: f32,
    drip: f32,
    drip_speed: f32,
    sats: [(Vec2, f32); 5],
}

/// Un texte rigolo dans le fil des messages.
struct Msg {
    title: String,
    sub: String,
    age: f32,
}

const MSG_LIFE: f32 = 6.0; // duree d'affichage d'un message seul
const MSG_MIN: f32 = 3.0; // temps de lecture minimum quand d'autres attendent
const MSG_FADE: f32 = 0.5; // fondu de sortie
const MSG_KEEP: usize = 5; // message affiche + messages en attente

struct Pop {
    pos: Vec2,
    t: f32,
    txt: String,
    col: Color,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Title,
    Running,
    Over,
}

#[derive(Clone, Copy, PartialEq)]
enum Death {
    Tram,
    KO,
    Mine,
}

struct Game {
    phase: Phase,
    time: f32,
    gore: bool,
    best: u32,
    new_record: bool,

    // Acaboulette
    pos: Vec2, // pieds
    vy: f32,
    speed: f32,
    on_ground: bool,
    jumps_left: u32,
    tank: f32, // temps restant dans le char d'assaut (0 = a pattes)
    tank_recoil: f32,
    shell_cd: f32,
    missile_cd: f32,
    next_tank_x: f32,
    tank_spots: Vec<TankSpot>,
    shells: Vec<Shell>,
    missiles: Vec<Missile>,
    glide_left: bool, // le deltaplane (3e saut) est encore disponible
    gliding: bool,
    glide_t: f32,   // vol consomme (les coups de nez en coutent)
    glide_age: f32, // temps depuis l'ouverture de l'aile
    holding: bool,
    slamming: bool,
    fat: f32,
    multi_n: u32,
    multi_t: f32,
    lives: u32,
    invuln: f32,
    hurt_t: f32,
    gave: f32,
    kebab: f32,
    closeup: f32,
    closeup_len: f32,
    closeup_kind: Closeup,
    first_gave: bool,
    run_phase: f32,
    squash: f32,
    bloody: f32,
    death: Option<Death>,
    over_t: f32,
    on_tram: bool,
    cam_y: f32,

    // score
    score: f32,
    combo: u32,
    combo_t: f32,
    kills: [u32; 5],
    gull_hint: bool, // les mouettes n'ont pas encore ete annoncees
    gave_count: u32,
    hazards: Vec<Hazard>,

    critters: Vec<Critter>,
    obstacles: Vec<Obstacle>,
    trams: Vec<Tram>,
    vehicles: Vec<Vehicle>,
    pickups: Vec<Pickup>,
    particles: Vec<Particle>,
    flung: Vec<Flung>,
    decals: Vec<Decal>,
    splats: Vec<Splat>,
    pops: Vec<Pop>,

    next_spawn: f32,
    no_obstacle_until: f32,
    zone_seen: i32,
    msgs: Vec<Msg>,
    tarpin_cd: f32,
    first_tram_kill: bool,
    first_aps: bool,
    shake_t: f32,
}

impl Game {
    fn new(best: u32, gore: bool) -> Game {
        let mut g = Game {
            phase: Phase::Title,
            time: 0.0,
            gore,
            best,
            new_record: false,
            pos: vec2(0.0, 0.0),
            vy: 0.0,
            speed: START_SPEED,
            on_ground: true,
            jumps_left: 1,
            tank: 0.0,
            tank_recoil: 0.0,
            shell_cd: 0.0,
            missile_cd: 0.0,
            next_tank_x: TANK_MIN_X,
            tank_spots: Vec::new(),
            shells: Vec::new(),
            missiles: Vec::new(),
            glide_left: true,
            gliding: false,
            glide_t: 0.0,
            glide_age: 0.0,
            holding: false,
            slamming: false,
            fat: 0.0,
            multi_n: 0,
            multi_t: 0.0,
            lives: LIVES,
            invuln: 0.0,
            hurt_t: 0.0,
            gave: 0.0,
            kebab: 0.0,
            closeup: 0.0,
            closeup_len: CLOSEUP_TIME,
            closeup_kind: Closeup::Kebab,
            first_gave: true,
            run_phase: 0.0,
            squash: 0.0,
            bloody: 0.0,
            death: None,
            over_t: 0.0,
            on_tram: false,
            cam_y: 0.0,
            score: 0.0,
            combo: 0,
            combo_t: 0.0,
            kills: [0; 5],
            gull_hint: true,
            gave_count: 0,
            hazards: Vec::new(),
            critters: Vec::new(),
            obstacles: Vec::new(),
            trams: Vec::new(),
            vehicles: Vec::new(),
            pickups: Vec::new(),
            particles: Vec::new(),
            flung: Vec::new(),
            decals: Vec::new(),
            splats: Vec::new(),
            pops: Vec::new(),
            next_spawn: 22.0,
            no_obstacle_until: 0.0,
            zone_seen: 0,
            msgs: Vec::new(),
            tarpin_cd: 0.0,
            first_tram_kill: true,
            first_aps: true,
            shake_t: 0.0,
        };
        // Un kebab offert des les premiers metres, a hauteur de moustaches
        g.pickups.push(Pickup { kind: PickKind::Kebab, pos: vec2(16.0, 0.9), taken: false, phase: 0.0 });
        // Quelques pigeons pour le decor de l'ecran titre
        for i in 0..4 {
            g.critters.push(Critter {
                kind: Kind::Pigeon,
                pos: vec2(4.0 + i as f32 * 1.3 + gen_range(-0.3, 0.3), 0.0),
                vel: vec2(0.0, 0.0),
                phase: gen_range(0.0, TAU),
                base_y: 0.0,
                flying: false,
                flag: true,
                dead: false,
            });
        }
        g
    }

    /// Indestructible : mode gave (trop gros), kebab turbo ou char d'assaut.
    fn mighty(&self) -> bool {
        self.gave > 0.0 || self.kebab > 0.0 || self.tank > 0.0
    }

    fn city(&self) -> City {
        zone_at(self.pos.x).city()
    }

    /// Le tram du coin.
    fn tram_name(&self) -> &'static str {
        match self.city() {
            City::Bordeaux => "tram B",
            City::Marseille => "petit train",
            City::Forcalquier => "tracteur",
            City::Algerie => "tram d'Alger",
        }
    }

    /// Quartier precedent, quartier courant et avancement du fondu entre les
    /// deux (couleur de l'eau, du ciel) sur les premiers metres du quartier.
    fn zone_fade(&self) -> (Zone, Zone, f32) {
        let x = self.pos.x;
        let cur = zone_at(x);
        if x < ZONE_LEN {
            return (cur, cur, 1.0); // premier quartier de la partie
        }
        let into = x - zone_index(x) as f32 * ZONE_LEN;
        (zone_at(x - ZONE_LEN), cur, (into / 45.0).min(1.0))
    }

    fn band_col(&self) -> Color {
        let (a, b, t) = self.zone_fade();
        mix(a.band(), b.band(), t)
    }

    fn sunny(&self) -> f32 {
        let (a, b, t) = self.zone_fade();
        a.sunny() + (b.sunny() - a.sunny()) * t
    }

    fn total(&self) -> u32 {
        self.score as u32 + self.pos.x.max(0.0) as u32
    }

    fn body_r(&self) -> f32 {
        0.3 + 0.085 * self.fat
    }

    /// Centre et rayon du cercle de collision de Acaboulette.
    fn hit_circle(&self) -> (Vec2, f32) {
        if self.tank > 0.0 {
            return (self.pos + vec2(0.7, 1.1), 1.5);
        }
        let r = self.body_r();
        (self.pos + vec2(0.1, 0.2 + r * 0.85), r * 0.9)
    }

    fn say(&mut self, txt: &str, sub: &str) {
        // un seul message a l'ecran ; les suivants attendent leur tour
        if let Some(m) = self.msgs.last()
            && m.title == txt
            && m.sub == sub
        {
            return;
        }
        self.msgs.push(Msg { title: txt.to_owned(), sub: sub.to_owned(), age: 0.0 });
        if self.msgs.len() > MSG_KEEP {
            self.msgs.remove(1); // on oublie le plus ancien en attente
        }
    }

    /// Les messages "tarpin" : de temps en temps seulement.
    fn say_tarpin(&mut self, txt: &str, sub: &str) -> bool {
        if self.tarpin_cd > 0.0 {
            return false;
        }
        self.say(txt, sub);
        self.tarpin_cd = 7.0;
        true
    }

    /// Le message affiche vieillit ; s'il y en a d'autres en attente, il cede
    /// la place (en fondu) apres un temps de lecture minimum.
    fn update_msgs(&mut self, dt: f32) {
        let waiting = self.msgs.len() > 1;
        if let Some(m) = self.msgs.first_mut() {
            m.age += dt;
            if waiting && m.age > MSG_MIN && m.age < MSG_LIFE - MSG_FADE {
                m.age = MSG_LIFE - MSG_FADE;
            }
            if m.age >= MSG_LIFE {
                self.msgs.remove(0);
            }
        }
    }

    fn pop(&mut self, pos: Vec2, txt: &str, col: Color) {
        self.pops.push(Pop { pos, t: 0.0, txt: txt.to_owned(), col });
    }

    fn shake(&mut self, t: f32) {
        self.shake_t = self.shake_t.max(t);
    }

    // -----------------------------------------------------------------------
    // Entrees
    // -----------------------------------------------------------------------

    fn input(&mut self) {
        if is_key_pressed(KeyCode::G) {
            self.gore = !self.gore;
        }
        let jump_pressed = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::W)
            || is_key_pressed(KeyCode::Z)
            || is_mouse_button_pressed(MouseButton::Left);
        self.holding = is_key_down(KeyCode::Space)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::W)
            || is_key_down(KeyCode::Z)
            || is_mouse_button_down(MouseButton::Left);
        let slam_pressed = is_key_pressed(KeyCode::Down)
            || is_key_pressed(KeyCode::S)
            || is_mouse_button_pressed(MouseButton::Right);

        match self.phase {
            Phase::Title => {
                // choix de la ville de depart : fleches, chiffres ou clic sur un bouton
                let n = CITIES.len();
                let mut sel = start_city();
                if is_key_pressed(KeyCode::Left) {
                    sel = (sel + n - 1) % n;
                }
                if is_key_pressed(KeyCode::Right) {
                    sel = (sel + 1) % n;
                }
                for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].into_iter().enumerate() {
                    if is_key_pressed(key) {
                        sel = k;
                    }
                }
                let mut start = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
                if is_mouse_button_pressed(MouseButton::Left)
                    && let Some(k) = city_button_at(mouse_position())
                {
                    sel = k;
                    start = true;
                }
                set_start_city(sel);
                if start {
                    self.phase = Phase::Running;
                    self.time = 0.0;
                    let z = zone_at(0.0);
                    self.say(z.name(), z.subtitle());
                }
            }
            Phase::Over => {
                if jump_pressed && self.over_t > 1.2 {
                    let best = self.best;
                    *self = Game::new(best, self.gore);
                    self.phase = Phase::Running;
                    self.critters.clear();
                    self.say(zone_at(0.0).name(), "C'est reparti, gros matou !");
                } else if is_key_pressed(KeyCode::V) && self.over_t > 0.6 {
                    // retour a l'ecran titre pour choisir une autre ville
                    let best = self.best;
                    *self = Game::new(best, self.gore);
                }
            }
            Phase::Running if self.closeup > 0.0 => {
                // gros plan : on peut passer apres un instant
                if jump_pressed && self.closeup_can_skip() {
                    self.closeup = 0.0;
                    self.end_closeup();
                }
            }
            Phase::Running if self.tank > 0.0 => {
                // dans le char : clic gauche = obus, clic droit = missiles
                if jump_pressed {
                    self.fire_shell();
                }
                if slam_pressed {
                    self.fire_missiles();
                }
            }
            Phase::Running => {
                if jump_pressed {
                    let jv = JUMP_V;
                    if self.on_ground {
                        self.vy = jv;
                        self.on_ground = false;
                        self.on_tram = false;
                        self.slamming = false;
                        self.dust(self.pos, 6);
                    } else if self.jumps_left > 0 {
                        self.jumps_left -= 1;
                        self.vy = jv * 0.85;
                        self.slamming = false;
                    } else if self.gliding {
                        // coup de nez : ca remonte un peu, de moins en moins
                        self.vy = self.vy.max(glide_lift(self.glide_t));
                        self.glide_t += GLIDE_FLARE_COST;
                    } else if self.glide_left {
                        // triple saut : il sort le deltaplane
                        self.glide_left = false;
                        self.gliding = true;
                        self.glide_t = 0.0;
                        self.glide_age = 0.0;
                        self.slamming = false;
                        self.vy = self.vy.max(2.0);
                        let (cc, _) = self.hit_circle();
                        self.pop(cc + vec2(0.0, 1.6), "FLAP !", Color::new(0.6, 0.9, 1.0, 1.0));
                    }
                }
                if slam_pressed && !self.on_ground && !self.slamming {
                    self.gliding = false;
                    self.slamming = true;
                    self.vy = SLAM_V;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Simulation
    // -----------------------------------------------------------------------

    fn step(&mut self, dt: f32) {
        self.time += dt;
        if self.closeup > 0.0 {
            // tout est fige pendant le gros plan, jusqu'au clic du joueur
            self.closeup = (self.closeup - dt).max(1.0e-4);
            return;
        }
        self.update_msgs(dt);
        self.tarpin_cd -= dt;
        self.shake_t = (self.shake_t - dt).max(0.0);
        self.squash = (self.squash - dt * 4.0).max(0.0);
        self.bloody = (self.bloody - dt * 0.03).max(0.0);
        let target = (self.pos.y - 3.6).max(0.0);
        self.cam_y += (target - self.cam_y) * (dt * 8.0).min(1.0);

        match self.phase {
            Phase::Title => {
                self.run_phase += dt * 2.0;
                self.update_critters(dt, false);
            }
            Phase::Running => self.step_running(dt),
            Phase::Over => {
                self.over_t += dt;
                self.update_critters(dt, false);
                self.update_trams(dt);
                self.update_vehicles(dt);
            }
        }
        self.update_particles(dt);
    }

    fn step_running(&mut self, dt: f32) {
        self.invuln = (self.invuln - dt).max(0.0);
        self.hurt_t = (self.hurt_t - dt).max(0.0);
        let had_gave = self.gave > 0.0;
        self.gave = (self.gave - dt).max(0.0);
        if had_gave && self.gave == 0.0 {
            self.fat = 0.5;
            self.invuln = self.invuln.max(1.0);
            self.say("Fin du mode gavé", "Tout est resté dans la litière : Acaboulette est re-mince");
        }
        if self.tank > 0.0 {
            self.tank -= dt;
            self.shell_cd = (self.shell_cd - dt).max(0.0);
            self.missile_cd = (self.missile_cd - dt).max(0.0);
            self.tank_recoil = (self.tank_recoil - dt * 5.0).max(0.0);
            if self.tank <= 0.0 {
                self.leave_tank();
            } else if gen_range(0.0, 1.0) < 0.12 {
                // la fumee du joint
                let tip = self.pos + vec2(-TANK_SEAT, 0.0) + TANK_JOINT_TIP;
                self.particles.push(Particle {
                    kind: PK::Dust,
                    pos: tip,
                    vel: vec2(gen_range(-1.5, 0.2), gen_range(1.2, 2.4)),
                    ang: 0.0,
                    spin: 0.0,
                    size: gen_range(0.05, 0.1),
                    col: Color::new(0.92, 0.95, 0.9, 0.6),
                    life: gen_range(0.6, 1.1),
                    max_life: 1.1,
                });
            }
        }
        if self.fat >= MAX_FAT && self.gave == 0.0 && self.tank == 0.0 {
            // Trop gros : pause litiere, puis mode gave
            self.closeup = LITTER_TIME;
            self.closeup_len = LITTER_TIME;
            self.closeup_kind = Closeup::Litter;
            self.slamming = false;
            return;
        }
        let had_kebab = self.kebab > 0.0;
        self.kebab = (self.kebab - dt).max(0.0);
        if had_kebab && self.kebab == 0.0 {
            self.say("Fin du kebab", "Retour à la vitesse normale. Tarpin dommage.");
            self.invuln = self.invuln.max(1.0);
        }
        self.combo_t -= dt;
        if self.combo_t <= 0.0 {
            self.combo = 0;
        }
        if self.multi_t > 0.0 {
            self.multi_t -= dt;
            if self.multi_t <= 0.0 {
                self.end_multi();
            }
        }

        self.speed = (self.speed + SPEED_GAIN * dt).min(MAX_SPEED);
        let spd = if self.tank > 0.0 {
            TANK_SPEED
        } else {
            self.speed * (1.0 - 0.45 * (self.hurt_t / 0.6))
        } * if self.tank > 0.0 {
            1.0
        } else if self.kebab > 0.0 {
            KEBAB_SPEED
        } else if self.gave > 0.0 {
            1.25
        } else {
            1.0
        };
        self.pos.x += spd * dt;
        if self.on_ground {
            self.run_phase += dt * spd * 1.5;
        } else {
            self.run_phase += dt * 3.0;
        }

        if self.tank > 0.0 {
            // le char reste au sol
            self.pos.y = 0.0;
            self.vy = 0.0;
            self.on_ground = true;
            self.on_tram = false;
        }

        // Saut / gravite / atterrissage
        let prev_y = self.pos.y;
        let r = self.body_r();
        let foot = r * 0.75;
        if !self.on_ground {
            if self.gliding {
                self.glide_t += dt;
                self.glide_age += dt;
                self.vy = (self.vy - GLIDE_GRAV * dt).max(-glide_sink(self.glide_t));
                if self.glide_t >= GLIDE_MAX {
                    self.gliding = false; // l'aile se replie : il retombe
                }
            } else {
                let g = if self.holding && self.vy > 0.0 && !self.slamming { GRAV * HOLD_FACTOR } else { GRAV };
                self.vy -= g * dt;
            }
            self.pos.y += self.vy * dt;
            if self.vy <= 0.0 {
                let mut top = 0.0f32;
                let mut tram_top = false;
                for o in &self.obstacles {
                    if o.knocked == 0.0
                        && !o.smashed
                        && self.pos.x + foot > o.x
                        && self.pos.x - foot < o.x + o.w
                        && prev_y >= o.h - 0.1
                        && o.h > top
                    {
                        top = o.h;
                    }
                }
                for v in &self.vehicles {
                    if v.kind == VehKind::Car
                        && !v.hit
                        && self.pos.x + foot > v.x + 0.5
                        && self.pos.x - foot < v.x + v.w() - 0.3
                        && prev_y >= v.h() - 0.15
                        && v.h() > top
                    {
                        top = v.h();
                    }
                }
                for t in &self.trams {
                    if self.pos.x + foot > t.x + 0.6
                        && self.pos.x - foot < t.x + TRAM_LEN
                        && prev_y >= TRAM_TOP - 0.15
                        && TRAM_TOP > top
                    {
                        top = TRAM_TOP;
                        tram_top = true;
                    }
                }
                if self.pos.y <= top {
                    self.pos.y = top;
                    self.land(tram_top);
                }
            }
        } else if self.pos.y > 0.0 {
            // Toujours quelque chose sous les pattes ?
            let mut supported = false;
            for o in &self.obstacles {
                if o.knocked == 0.0
                    && !o.smashed
                    && (self.pos.y - o.h).abs() < 0.05
                    && self.pos.x + foot > o.x
                    && self.pos.x - foot < o.x + o.w
                {
                    supported = true;
                }
            }
            for v in &self.vehicles {
                if v.kind == VehKind::Car
                    && !v.hit
                    && (self.pos.y - v.h()).abs() < 0.05
                    && self.pos.x - foot < v.x + v.w()
                    && self.pos.x + foot > v.x
                {
                    supported = true;
                }
            }
            for t in &self.trams {
                if (self.pos.y - TRAM_TOP).abs() < 0.05 && self.pos.x - foot < t.x + TRAM_LEN && self.pos.x + foot > t.x {
                    supported = true;
                }
            }
            if !supported {
                self.on_ground = false;
                self.on_tram = false;
                self.vy = 0.0;
            }
        }

        // Eclaboussures sur le Miroir d'eau
        if self.on_ground && self.pos.y == 0.0 && zone_at(self.pos.x) == Zone::Bourse && gen_range(0.0, 1.0) < 0.35 {
            let p = self.pos + vec2(gen_range(-0.4, 0.4), 0.02);
            self.particles.push(Particle {
                kind: PK::Water,
                pos: p,
                vel: vec2(gen_range(-3.0, 1.0), gen_range(1.5, 3.5)),
                ang: 0.0,
                spin: 0.0,
                size: gen_range(0.03, 0.06),
                col: Color::new(0.75, 0.85, 0.95, 0.8),
                life: 1.0,
                max_life: 1.0,
            });
        }

        self.update_trams(dt);
        self.update_vehicles(dt);
        self.update_critters(dt, true);
        self.collide_obstacles();
        if self.phase != Phase::Running {
            return;
        }
        self.collide_vehicles();
        if self.phase != Phase::Running {
            return;
        }
        self.collide_hazards(dt);
        if self.phase != Phase::Running {
            return;
        }
        self.collide_trams();
        if self.phase != Phase::Running {
            return;
        }
        self.collide_critters();
        self.collide_pickups();
        self.enter_tank();
        self.update_projectiles(dt);

        // Nouveau quartier
        let zi = zone_index(self.pos.x);
        if zi != self.zone_seen {
            self.zone_seen = zi;
            let z = zone_at(self.pos.x);
            self.say(z.name(), z.subtitle());
        }

        // Generation devant
        while self.next_spawn < self.pos.x + SPAWN_AHEAD {
            self.spawn_pattern();
        }

        // Menage derriere
        let back = self.pos.x - 16.0;
        self.critters.retain(|c| !c.dead && c.pos.x > back && c.pos.y < 14.0);
        self.obstacles.retain(|o| o.x + o.w > back);
        self.trams.retain(|t| t.x + TRAM_LEN > back - 10.0);
        self.vehicles.retain(|v| v.x + v.w() > back - 10.0);
        self.hazards.retain(|h| h.x > back);
        self.tank_spots.retain(|t| t.x + TANK_LEN > back);
        self.pickups.retain(|p| !p.taken && p.pos.x > back);
        self.decals.retain(|d| d.x > back);
    }

    fn land(&mut self, tram: bool) {
        self.on_ground = true;
        self.jumps_left = 1;
        self.glide_left = true;
        self.gliding = false;
        self.squash = if self.slamming { 1.0 } else { 0.5 };
        self.on_tram = tram;
        if tram && self.first_aps {
            self.first_aps = false;
            match self.city() {
                City::Bordeaux => self.say("Sur le toit du tram B !", "Pas de caténaire en centre-ville : c'est l'APS, peuchère"),
                City::Marseille => self.say("Sur le toit du petit train !", "Visite guidée du Vieux-Port, et sans payer le ticket"),
                City::Forcalquier => self.say("Sur les bottes de foin !", "Ça gratte, mais on voit la montagne de Lure"),
                City::Algerie => self.say("Sur le toit du tram d'Alger !", "Vue sur la baie, et sans payer le ticket"),
            }
        }
        if self.slamming {
            self.slamming = false;
            self.shockwave();
        } else {
            self.dust(self.pos, 4);
        }
        self.vy = 0.0;
    }

    /// Le plat ventre : onde de choc qui ecrabouille tout autour.
    fn shockwave(&mut self) {
        let radius = 2.0 + 0.25 * self.fat;
        let c = self.pos;
        self.shake(0.35);
        self.pop(c + vec2(0.0, 1.6), "BADABOUM !", Color::new(1.0, 0.8, 0.1, 1.0));
        self.dust(c, 22);
        let mut victims = Vec::new();
        for (i, cr) in self.critters.iter().enumerate() {
            if !cr.dead && (cr.pos.x - c.x).abs() < radius && cr.pos.y - c.y < 1.4 && cr.pos.y >= c.y - 0.5 {
                victims.push(i);
            }
        }
        for i in victims {
            self.kill(i, true);
        }
        if self.mighty() {
            for k in 0..self.obstacles.len() {
                let o = &self.obstacles[k];
                if !o.smashed && (o.x + o.w / 2.0 - c.x).abs() < radius + o.w / 2.0 {
                    self.smash_obstacle(k);
                }
            }
        }
    }

    fn dust(&mut self, at: Vec2, n: usize) {
        for _ in 0..n {
            self.particles.push(Particle {
                kind: PK::Dust,
                pos: at + vec2(gen_range(-0.3, 0.3), 0.05),
                vel: vec2(gen_range(-4.0, 4.0), gen_range(0.2, 1.5)),
                ang: 0.0,
                spin: 0.0,
                size: gen_range(0.12, 0.3),
                col: Color::new(0.85, 0.8, 0.7, 0.6),
                life: 0.6,
                max_life: 0.6,
            });
        }
    }

    fn update_trams(&mut self, dt: f32) {
        let cat_x = self.pos.x;
        let mut warn = false;
        for t in &mut self.trams {
            t.x -= TRAM_SPEED * dt;
            if !t.warned && t.x - cat_x < 34.0 {
                t.warned = true;
                warn = true;
            }
        }
        if warn && self.phase == Phase::Running {
            let title = match self.city() {
                City::Bordeaux => "DING DING ! Tram B !",
                City::Marseille => "TCHOU TCHOU ! Le petit train des touristes !",
                City::Forcalquier => "POUET ! Un tracteur et son foin !",
                City::Algerie => "DING DING ! Tram d'Alger !",
            };
            let sub = if self.city() == City::Forcalquier { "Saute sur les bottes de foin, ou finis en crêpe" } else { "Saute sur le toit, ou finis en crêpe" };
            self.say(title, sub);
        }
        // Le tram ecrase ce qui traine sur les rails
        let mut victims = Vec::new();
        for t in &self.trams {
            for (i, c) in self.critters.iter().enumerate() {
                if !c.dead && !c.flying && c.pos.y < 0.5 && c.pos.x > t.x - 0.2 && c.pos.x < t.x + 0.4 {
                    victims.push(i);
                }
            }
        }
        if !victims.is_empty() {
            for i in victims {
                self.kill(i, false);
            }
            if self.first_tram_kill && self.phase == Phase::Running {
                self.first_tram_kill = false;
                let title = format!("Le {} fait le ménage", self.tram_name());
                self.say_tarpin(&title, "Tarpin efficace, pour une fois qu'il est à l'heure");
            }
        }
    }

    fn update_vehicles(&mut self, dt: f32) {
        let cat_x = self.pos.x;
        let mut honks = Vec::new();
        for v in &mut self.vehicles {
            v.x -= v.speed * dt;
            if !v.honked && v.x - cat_x < 16.0 {
                v.honked = true;
                honks.push((v.kind, vec2(v.x, v.h() + 0.6)));
            }
        }
        if self.phase == Phase::Running {
            for (k, at) in honks {
                let t = match k {
                    VehKind::Car => pick(&["POUET POUET !", "TUUUT !"]),
                    VehKind::Moto => pick(&["VROOOAAR !", "BRAAAP !"]),
                };
                self.pop(at, t, Color::new(1.0, 0.6, 0.2, 1.0));
            }
        }
        // Les vehicules ecrasent les bestioles sur leur passage
        let mut victims = Vec::new();
        for v in &self.vehicles {
            for (i, c) in self.critters.iter().enumerate() {
                if !c.dead && !c.flying && c.pos.y < 0.5 && c.pos.x > v.x - 0.2 && c.pos.x < v.x + 0.4 {
                    victims.push(i);
                }
            }
        }
        for i in victims {
            self.kill(i, false);
        }
    }

    fn collide_vehicles(&mut self) {
        let (cc, cr) = self.hit_circle();
        for k in 0..self.vehicles.len() {
            let v = &self.vehicles[k];
            if v.hit {
                continue;
            }
            let (w, h) = (v.w(), v.h());
            if v.kind == VehKind::Car && self.pos.y >= h - 0.15 {
                continue; // sur le toit
            }
            let near = vec2(cc.x.clamp(v.x, v.x + w), cc.y.clamp(0.0, h));
            if near.distance(cc) >= cr * 0.8 {
                continue;
            }
            let kind = v.kind;
            self.vehicles[k].hit = true;
            if self.mighty() {
                // mode gave / kebab / char : le vehicule part en morceaux
                self.wreck_vehicle(k);
                if self.tank == 0.0 {
                    self.say_tarpin("Carrosserie en kit", "Mode gavé : même les voitures y passent. Tarpin !");
                }
            } else {
                let line = match kind {
                    VehKind::Car => pick(&["Renversé par une voiture !", "Un SUV sur les quais : tarpin mal", "Bouchon sur la rocade, et toi dessous"]),
                    VehKind::Moto => pick(&["Fauché par une moto !", "Un livreur à scooter, classique", "La moto ne s'est pas arrêtée"]),
                };
                self.hurt(line);
                if self.phase != Phase::Running {
                    return;
                }
            }
        }
    }

    /// Le vehicule part en morceaux.
    fn wreck_vehicle(&mut self, k: usize) {
        let v = &mut self.vehicles[k];
        let (w, h, col) = (v.w(), v.h(), v.color);
        let mid = vec2(v.x + w / 2.0, h / 2.0);
        v.hit = true;
        v.x = -1.0e6;
        for i in 0..22 {
            self.particles.push(Particle {
                kind: PK::Debris,
                pos: mid + vec2(gen_range(-w / 2.0, w / 2.0), gen_range(-0.4, 0.4)),
                vel: vec2(gen_range(2.0, 10.0), gen_range(3.0, 10.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-12.0, 12.0),
                size: gen_range(0.1, 0.28),
                col: if i % 3 == 0 { INK } else { col },
                life: 4.0,
                max_life: 4.0,
            });
        }
        self.score += 150.0;
        self.shake(0.4);
        self.pop(mid + vec2(0.0, 1.2), "KABOUM ! +150", Color::new(1.0, 0.8, 0.1, 1.0));
    }

    /// Le tram B pulverise (par le char ou son obus).
    fn wreck_tram(&mut self, k: usize) {
        let x = self.trams[k].x;
        self.trams[k].x = -1.0e6;
        for i in 0..90 {
            let col = match i % 4 {
                0 => INK,
                1 => Color::new(0.85, 0.87, 0.9, 1.0),
                2 => Color::new(0.2, 0.35, 0.6, 1.0),
                _ => Color::new(0.55, 0.6, 0.66, 1.0),
            };
            self.particles.push(Particle {
                kind: PK::Debris,
                pos: vec2(x + gen_range(0.0, TRAM_LEN), gen_range(0.3, TRAM_TOP)),
                vel: vec2(gen_range(-2.0, 14.0), gen_range(3.0, 13.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-14.0, 14.0),
                size: gen_range(0.12, 0.4),
                col,
                life: 4.0,
                max_life: 4.0,
            });
        }
        for i in 0..4 {
            self.explosion(vec2(x + 1.0 + i as f32 * 4.5, 1.4));
        }
        self.score += 500.0;
        self.shake(0.9);
        let at = vec2(x.max(self.pos.x + 2.0).min(self.pos.x + 9.0), TRAM_TOP + 1.2);
        self.pop(at, "PULVÉRISÉ ! +500", Color::new(1.0, 0.8, 0.1, 1.0));
        let title = format!("Le {} pulvérisé !", self.tram_name());
        self.say(&title, "Trafic interrompu sur la ligne. Tarpin longtemps.");
    }

    // -----------------------------------------------------------------------
    // Le char d'assaut
    // -----------------------------------------------------------------------

    /// Il faut sauter dans le char a l'arret pour le prendre.
    fn enter_tank(&mut self) {
        if self.tank > 0.0 || self.on_ground {
            return;
        }
        // en l'air au-dessus du char (meme d'assez haut) : il tombe dedans
        let (cc, cr) = self.hit_circle();
        let Some(k) = self.tank_spots.iter().position(|t| {
            let near = vec2(cc.x.clamp(t.x, t.x + TANK_LEN), cc.y.clamp(0.0, TANK_H + 3.5));
            near.distance(cc) < cr
        }) else {
            return;
        };
        let spot = self.tank_spots.remove(k);
        self.tank = TANK_TIME;
        self.shell_cd = 0.0;
        self.missile_cd = 0.0;
        self.tank_recoil = 0.0;
        self.pos = vec2(spot.x + TANK_SEAT, 0.0);
        self.vy = 0.0;
        self.on_ground = true;
        self.on_tram = false;
        self.gliding = false;
        self.slamming = false;
        self.jumps_left = 1;
        self.glide_left = true;
        self.shake(0.3);
        self.dust(self.pos, 14);
        self.pop(self.pos + vec2(0.5, 3.2), "VROUM !", Color::new(0.6, 1.0, 0.4, 1.0));
        self.say("CHAR D'ASSAUT !", "Clic gauche : OBUS  -  Clic droit : MISSILES");
    }

    /// Plus de carburant : le char saute, Acaboulette est ejecte.
    fn leave_tank(&mut self) {
        self.tank = 0.0;
        let at = self.pos + vec2(0.5, 1.0);
        self.explosion(at);
        for i in 0..26 {
            self.particles.push(Particle {
                kind: PK::Debris,
                pos: at + vec2(gen_range(-1.8, 2.2), gen_range(-0.6, 0.8)),
                vel: vec2(gen_range(-3.0, 9.0), gen_range(3.0, 11.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-12.0, 12.0),
                size: gen_range(0.12, 0.35),
                col: if i % 3 == 0 { INK } else { TANK_GREEN },
                life: 4.0,
                max_life: 4.0,
            });
        }
        self.vy = JUMP_V;
        self.on_ground = false;
        self.jumps_left = 1;
        self.glide_left = true;
        self.invuln = self.invuln.max(2.0);
        self.say("Le char est à sec !", "Plus une goutte de gazole : retour aux pattes");
    }

    /// Bout du canon.
    fn muzzle(&self) -> Vec2 {
        vec2(self.pos.x - TANK_SEAT + 5.7, 1.72)
    }

    fn fire_shell(&mut self) {
        if self.shell_cd > 0.0 {
            return;
        }
        self.shell_cd = SHELL_CD;
        self.tank_recoil = 1.0;
        let m = self.muzzle();
        self.shells.push(Shell { pos: m });
        self.shake(0.3);
        self.pop(m + vec2(0.5, 1.0), "BLAM !", Color::new(1.0, 0.8, 0.1, 1.0));
        for i in 0..12 {
            self.particles.push(Particle {
                kind: PK::Dust,
                pos: m,
                vel: vec2(gen_range(4.0, 14.0), gen_range(-2.5, 2.5)),
                ang: 0.0,
                spin: 0.0,
                size: gen_range(0.15, 0.4),
                col: if i % 2 == 0 { Color::new(1.0, 0.85, 0.2, 0.9) } else { Color::new(1.0, 0.45, 0.05, 0.9) },
                life: gen_range(0.2, 0.45),
                max_life: 0.45,
            });
        }
    }

    fn fire_missiles(&mut self) {
        if self.missile_cd > 0.0 {
            return;
        }
        self.missile_cd = MISSILE_CD;
        let targets = self.targets();
        let rack = vec2(self.pos.x - TANK_SEAT + 1.5, 2.4);
        for i in 0..MISSILE_N {
            let aim = if targets.is_empty() { None } else { Some(targets[i % targets.len()]) };
            let k = i as f32;
            self.missiles.push(Missile {
                pos: rack + vec2(k * 0.08, k * 0.05),
                vel: vec2(TANK_SPEED + 1.0 + k * 1.6, 7.0 + k * 1.7),
                aim,
                age: 0.0,
            });
        }
        self.pop(rack + vec2(0.0, 1.4), "FCHHHT !", Color::new(1.0, 1.0, 1.0, 1.0));
    }

    /// Tout ce qu'un missile peut viser devant le char : obstacles, vehicules,
    /// mines et bestioles (du plus proche au plus lointain).
    fn targets(&self) -> Vec<Vec2> {
        let (x0, x1) = (self.pos.x + 1.0, self.pos.x + 19.0);
        let mut out = Vec::new();
        for o in &self.obstacles {
            if !o.smashed && o.knocked == 0.0 {
                out.push(vec2(o.x + o.w / 2.0, o.h / 2.0));
            }
        }
        for v in &self.vehicles {
            if !v.hit {
                out.push(vec2(v.x + v.w() / 2.0, v.h() / 2.0));
            }
        }
        for h in &self.hazards {
            if !h.fired {
                out.push(vec2(h.x, 0.15));
            }
        }
        for c in &self.critters {
            if !c.dead {
                out.push(c.center());
            }
        }
        out.retain(|t| t.x > x0 && t.x < x1);
        out.sort_by(|a, b| a.x.total_cmp(&b.x));
        out
    }

    /// Detruit tout ce qui se trouve dans la boite. Renvoie vrai si un tram
    /// a ete pulverise (seulement si `trams`).
    fn blast(&mut self, x0: f32, x1: f32, y0: f32, y1: f32, trams: bool) -> bool {
        for k in 0..self.obstacles.len() {
            let o = &self.obstacles[k];
            if !o.smashed && o.knocked == 0.0 && o.x < x1 && o.x + o.w > x0 && y0 < o.h {
                self.smash_obstacle(k);
            }
        }
        for k in 0..self.vehicles.len() {
            let v = &self.vehicles[k];
            if !v.hit && v.x < x1 && v.x + v.w() > x0 && y0 < v.h() {
                self.wreck_vehicle(k);
            }
        }
        for k in 0..self.hazards.len() {
            let h = &self.hazards[k];
            if !h.fired && h.x > x0 - 0.3 && h.x < x1 + 0.3 && y0 < 0.4 {
                let at = vec2(h.x, 0.1);
                self.hazards[k].fired = true;
                self.explosion(at);
                self.score += 100.0;
            }
        }
        let victims: Vec<usize> = self
            .critters
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                let (p, r) = (c.center(), c.kind.radius());
                !c.dead && p.x > x0 - r && p.x < x1 + r && p.y > y0 - r && p.y < y1 + r
            })
            .map(|(i, _)| i)
            .collect();
        for i in victims {
            self.kill(i, true);
        }
        let mut tram_hit = false;
        if trams {
            for k in 0..self.trams.len() {
                let tx = self.trams[k].x;
                if tx < x1 && tx + TRAM_LEN > x0 && y0 < TRAM_TOP {
                    self.wreck_tram(k);
                    tram_hit = true;
                }
            }
        }
        tram_hit
    }

    /// Petite explosion d'un missile.
    fn small_boom(&mut self, at: Vec2) {
        self.shake(0.12);
        for i in 0..14 {
            let a = gen_range(0.0, TAU);
            self.particles.push(Particle {
                kind: PK::Dust,
                pos: at,
                vel: Vec2::from_angle(a) * gen_range(1.0, 6.0),
                ang: 0.0,
                spin: 0.0,
                size: gen_range(0.15, 0.35),
                col: match i % 3 {
                    0 => Color::new(1.0, 0.85, 0.2, 0.9),
                    1 => Color::new(1.0, 0.45, 0.05, 0.9),
                    _ => Color::new(0.3, 0.3, 0.3, 0.7),
                },
                life: gen_range(0.3, 0.6),
                max_life: 0.6,
            });
        }
    }

    fn smoke(&mut self, at: Vec2, size: f32) {
        self.particles.push(Particle {
            kind: PK::Dust,
            pos: at,
            vel: vec2(gen_range(-0.5, 0.5), gen_range(-0.3, 0.6)),
            ang: 0.0,
            spin: 0.0,
            size,
            col: Color::new(0.85, 0.85, 0.85, 0.55),
            life: 0.45,
            max_life: 0.45,
        });
    }

    fn update_projectiles(&mut self, dt: f32) {
        // Obus : tout droit, il traverse tout (sauf le tram, qu'il pulverise)
        let mut shells = std::mem::take(&mut self.shells);
        let limit = self.pos.x + 24.0;
        shells.retain_mut(|sh| {
            let prev = sh.pos.x;
            sh.pos.x += SHELL_SPEED * dt;
            if gen_range(0.0, 1.0) < 0.5 {
                self.smoke(sh.pos - vec2(0.7, 0.0), 0.16);
            }
            let tram_hit = self.blast(prev - 0.4, sh.pos.x + 0.6, 0.0, sh.pos.y + 1.4, true);
            !tram_hit && sh.pos.x < limit
        });
        self.shells = shells;

        // Missiles a tete chercheuse
        let mut missiles = std::mem::take(&mut self.missiles);
        missiles.retain_mut(|m| {
            m.age += dt;
            // la tete chercheuse suit sa cible, ou en cherche une autre
            let targets = self.targets();
            let near = |p: Vec2| targets.iter().copied().min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
            m.aim = match m.aim.and_then(near) {
                Some(t) if t.distance(m.aim.unwrap_or(t)) < 3.5 => Some(t),
                _ => near(m.pos),
            };
            let speed = (14.0 + m.age * 34.0).min(36.0);
            let dir = match m.aim {
                Some(a) => (a - m.pos).normalize_or_zero(),
                None => vec2(1.0, -0.12).normalize(),
            };
            m.vel = m.vel.lerp(dir * speed, (dt * (3.0 + m.age * 14.0)).min(1.0));
            m.pos += m.vel * dt;
            if gen_range(0.0, 1.0) < 0.4 {
                self.smoke(m.pos - m.vel.normalize_or_zero() * 0.4, 0.1);
            }
            let reached = m.aim.is_some_and(|a| a.distance(m.pos) < 0.7);
            if reached || m.pos.y < 0.12 || m.age > 2.2 {
                self.small_boom(m.pos);
                self.blast(m.pos.x - 1.0, m.pos.x + 1.0, m.pos.y - 1.0, m.pos.y + 1.0, false);
                return false;
            }
            true
        });
        self.missiles = missiles;
    }

    fn collide_hazards(&mut self, dt: f32) {
        for h in &mut self.hazards {
            if h.fired {
                h.t += dt;
            }
        }
        if self.pos.y > 0.12 {
            return; // il faut avoir les pattes au sol
        }
        let foot = self.body_r() * 0.5;
        for k in 0..self.hazards.len() {
            let (fired, hx, kind) = (self.hazards[k].fired, self.hazards[k].x, self.hazards[k].kind);
            if fired || (self.pos.x - hx).abs() > kind.half_w() + foot {
                continue;
            }
            self.hazards[k].fired = true;
            let at = vec2(hx, 0.1);
            match kind {
                HazKind::Mine => {
                    self.explosion(at);
                    if self.tank > 0.0 {
                        self.score += 100.0;
                        self.pop(at + vec2(0.0, 2.6), "Même pas une rayure ! +100", Color::new(1.0, 0.85, 0.2, 1.0));
                    } else if self.mighty() || self.invuln > 0.0 {
                        self.vy = JUMP_V * 1.3;
                        self.on_ground = false;
                        self.on_tram = false;
                        self.score += 100.0;
                        self.pop(at + vec2(0.0, 2.2), "Même pas mal ! +100", Color::new(1.0, 0.85, 0.2, 1.0));
                    } else {
                        self.die(Death::Mine);
                        return;
                    }
                }
            }
        }
    }

    /// Boule de feu, fumee et secousse.
    fn explosion(&mut self, at: Vec2) {
        self.shake(0.7);
        self.pop(at + vec2(0.0, 1.5), "BOUM !", Color::new(1.0, 0.6, 0.1, 1.0));
        for i in 0..40 {
            let a = gen_range(0.2, PI - 0.2);
            let sp = gen_range(2.0, 9.0);
            let col = match i % 3 {
                0 => Color::new(1.0, 0.85, 0.2, 0.9),
                1 => Color::new(1.0, 0.45, 0.05, 0.9),
                _ => Color::new(0.3, 0.3, 0.3, 0.7),
            };
            self.particles.push(Particle {
                kind: PK::Dust,
                pos: at,
                vel: Vec2::from_angle(a) * sp,
                ang: 0.0,
                spin: 0.0,
                size: gen_range(0.2, 0.5),
                col,
                life: gen_range(0.5, 1.0),
                max_life: 1.0,
            });
        }
        for _ in 0..10 {
            self.particles.push(Particle {
                kind: PK::Debris,
                pos: at,
                vel: vec2(gen_range(-5.0, 5.0), gen_range(4.0, 10.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-15.0, 15.0),
                size: gen_range(0.05, 0.12),
                col: Color::new(0.25, 0.3, 0.2, 1.0),
                life: 3.0,
                max_life: 3.0,
            });
        }
    }

    /// Acaboulette en morceaux.
    fn explode_cat(&mut self) {
        let (cc, cr) = self.hit_circle();
        let scale = 1.0 + cr;
        for i in 0..(160.0 * scale) as usize {
            let a = gen_range(0.0, TAU);
            self.particles.push(Particle {
                kind: PK::Blood,
                pos: cc + Vec2::from_angle(a) * gen_range(0.0, cr),
                vel: vec2(a.cos() * gen_range(1.0, 9.0), a.sin().abs() * gen_range(3.0, 13.0)),
                ang: 0.0,
                spin: 0.0,
                size: gen_range(0.03, 0.09),
                col: blood((i % 7) as f32 / 7.0),
                life: 4.0,
                max_life: 4.0,
            });
        }
        let cols = [GINGER, GINGER, GINGER_DARK, CREAM, Color::new(0.75, 0.12, 0.15, 1.0), BONE, NOSE];
        for i in 0..(22.0 * scale) as usize {
            self.particles.push(Particle {
                kind: PK::Chunk,
                pos: cc + vec2(gen_range(-cr, cr), gen_range(-cr, cr) * 0.6),
                vel: vec2(gen_range(-7.0, 7.0), gen_range(5.0, 14.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-14.0, 14.0),
                size: gen_range(0.1, 0.22) * (0.7 + cr * 0.6),
                col: cols[i % cols.len()],
                life: 30.0,
                max_life: 30.0,
            });
        }
        for _ in 0..2 {
            self.particles.push(Particle {
                kind: PK::Eye,
                pos: cc + vec2(0.3, 0.3),
                vel: vec2(gen_range(-4.0, 4.0), gen_range(8.0, 13.0)),
                ang: 0.0,
                spin: gen_range(-8.0, 8.0),
                size: 0.11,
                col: WHITE,
                life: 30.0,
                max_life: 30.0,
            });
        }
        for _ in 0..30 {
            self.particles.push(Particle {
                kind: PK::Fur,
                pos: cc,
                vel: vec2(gen_range(-6.0, 6.0), gen_range(2.0, 9.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-5.0, 5.0),
                size: gen_range(0.06, 0.13),
                col: if gen_range(0.0, 1.0) < 0.6 { GINGER } else { CREAM },
                life: 4.0,
                max_life: 4.0,
            });
        }
        // le sang sur l'ecran : seulement pour la mine et le tram
        for _ in 0..4 {
            self.add_splat();
        }
    }

    fn update_critters(&mut self, dt: f32, active: bool) {
        let cat_x = self.pos.x;
        let mut woof = Vec::new();
        let mut crs_shout = false;
        let mut gull_seen = false;
        for c in &mut self.critters {
            if c.dead {
                continue;
            }
            c.phase += dt;
            match c.kind {
                Kind::Pigeon => {
                    if c.flying {
                        c.vel.y += 2.0 * dt;
                        c.pos += c.vel * dt;
                    } else {
                        c.pos.x += c.vel.x * dt;
                        if active && !c.flag && c.pos.x - cat_x < 3.2 {
                            c.flag = true;
                            if gen_range(0.0, 1.0) < 0.3 {
                                c.flying = true;
                                c.vel = vec2(gen_range(0.5, 3.0), gen_range(3.0, 4.5));
                            }
                        }
                    }
                }
                Kind::Crs => {
                    c.pos.x += c.vel.x * dt;
                    if active && !c.flag && c.pos.x - cat_x < 12.0 && c.pos.x > cat_x {
                        c.flag = true;
                        crs_shout = true;
                    }
                }
                Kind::Rat => {
                    c.pos.x += c.vel.x * dt;
                }
                Kind::Gull => {
                    c.pos.x += c.vel.x * dt;
                    c.pos.y = c.base_y + (c.phase * 1.6).sin() * 0.3;
                    if active && !c.flag && c.pos.x - cat_x < 16.0 && c.pos.x > cat_x {
                        c.flag = true;
                        gull_seen = true;
                    }
                }
                Kind::Dog => {
                    c.pos.x += c.vel.x * dt;
                    if active && !c.flag && c.pos.x - cat_x < 6.0 && c.pos.x > cat_x {
                        c.flag = true;
                        woof.push(c.center() + vec2(-0.2, 0.6));
                    }
                }
            }
        }
        if gull_seen && self.gull_hint {
            self.gull_hint = false;
            let title = match self.city() {
                City::Marseille => "Des gabians là-haut !",
                City::Forcalquier => "Des mouettes à Forcalquier ?!",
                _ => "Des mouettes là-haut !",
            };
            self.say(title, "Trop haut pour un saut : sors le deltaplane");
        }
        for at in woof {
            self.pop(at, pick(&["WAF !", "WOUAF !", "GRRR !", "KAI KAI !"]), WHITE);
        }
        if crs_shout {
            let (a, b) = [
                ("BONUS CRS !", "Fonce dans le barrage, c'est tarpin de points"),
                ("Barrage de CRS !", "BONUS : plat ventre sur les boucliers !"),
                ("Les CRS sur les quais !", "BONUS : fais-les valdinguer !"),
            ][gen_range(0, 3)];
            self.say(a, b);
        }
    }

    fn collide_obstacles(&mut self) {
        let (cc, cr) = self.hit_circle();
        for k in 0..self.obstacles.len() {
            let o = &self.obstacles[k];
            if o.smashed || o.knocked > 0.0 {
                continue;
            }
            if self.pos.y >= o.h - 0.12 {
                continue; // dessus
            }
            let near = vec2(cc.x.clamp(o.x, o.x + o.w), cc.y.clamp(0.0, o.h));
            if near.distance(cc) < cr * 0.85 {
                if self.mighty() {
                    self.smash_obstacle(k);
                } else {
                    let kind = o.kind;
                    self.obstacles[k].knocked = 0.01;
                    let line = match kind {
                        ObsKind::Barrier => match self.city() {
                            City::Bordeaux => pick(&["Encore des travaux à Bordeaux...", "Travaux. Comme d'hab."]),
                            City::Marseille => pick(&["Encore des travaux à Marseille...", "Travaux. Depuis 2600 ans."]),
                            City::Forcalquier => pick(&["Des travaux, même à Forcalquier", "Travaux. Reprise après la sieste."]),
                            City::Algerie => pick(&["Encore des travaux à Alger...", "Travaux. Comme partout."]),
                        },
                        ObsKind::Bike => "Un vélo couché en travers, classique",
                        ObsKind::Bin => "La poubelle de la rue ! Tarpin dur",
                    };
                    self.hurt(line);
                }
            }
        }
        for o in &mut self.obstacles {
            if o.knocked > 0.0 && o.knocked < PI / 2.0 {
                o.knocked = (o.knocked + 0.08).min(PI / 2.0);
            }
        }
    }

    fn smash_obstacle(&mut self, k: usize) {
        let o = &mut self.obstacles[k];
        o.smashed = true;
        let (cx, h, kind) = (o.x + o.w / 2.0, o.h, o.kind);
        let cols: [Color; 2] = match kind {
            ObsKind::Barrier => [Color::new(0.9, 0.1, 0.1, 1.0), WHITE],
            ObsKind::Bike => [Color::new(0.75, 0.85, 0.92, 1.0), Color::new(0.2, 0.2, 0.22, 1.0)],
            ObsKind::Bin => [Color::new(0.2, 0.45, 0.25, 1.0), Color::new(0.15, 0.3, 0.18, 1.0)],
        };
        for i in 0..14 {
            self.particles.push(Particle {
                kind: PK::Debris,
                pos: vec2(cx + gen_range(-0.4, 0.4), gen_range(0.1, h)),
                vel: vec2(gen_range(2.0, 9.0), gen_range(2.0, 8.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-12.0, 12.0),
                size: gen_range(0.08, 0.2),
                col: cols[i % 2],
                life: 3.0,
                max_life: 3.0,
            });
        }
        self.score += 40.0;
        self.shake(0.15);
        self.pop(vec2(cx, h + 0.6), "KRAAK !", WHITE);
    }

    fn collide_trams(&mut self) {
        let (cc, cr) = self.hit_circle();
        for k in 0..self.trams.len() {
            let t = &self.trams[k];
            if self.pos.y >= TRAM_TOP - 0.15 {
                if self.on_tram && !t.rode && self.pos.x > t.x + 2.0 && self.pos.x < t.x + TRAM_LEN {
                    self.trams[k].rode = true;
                    self.score += 100.0;
                    self.pop(self.pos + vec2(0.0, 2.0), "+100 SURF !", Color::new(0.5, 0.9, 1.0, 1.0));
                }
                continue;
            }
            let near = vec2(cc.x.clamp(t.x, t.x + TRAM_LEN), cc.y.clamp(0.15, TRAM_TOP));
            if near.distance(cc) < cr * 0.8 {
                if self.tank > 0.0 {
                    self.wreck_tram(k);
                } else if self.mighty() {
                    self.vy = JUMP_V * 1.3;
                    self.on_ground = false;
                    self.trams[k].rode = true;
                    self.shake(0.4);
                    self.pop(cc + vec2(0.8, 1.0), "BOING !", Color::new(1.0, 0.8, 0.1, 1.0));
                    let title = format!("Coup de boule dans le {} !", self.tram_name());
                    self.say(&title, "Retard estimé : tarpin longtemps");
                } else {
                    self.die(Death::Tram);
                    return;
                }
            }
        }
    }

    fn collide_critters(&mut self) {
        let (cc, cr) = self.hit_circle();
        let mut kills = Vec::new();
        let mut stomp = false;
        for (i, c) in self.critters.iter().enumerate() {
            if c.dead {
                continue;
            }
            // Les mouettes ne s'attrapent qu'en deltaplane
            if c.kind == Kind::Gull && !self.gliding {
                continue;
            }
            let r = c.kind.radius();
            if c.center().distance(cc) < cr + r * 0.9 {
                kills.push(i);
                if c.kind != Kind::Gull && self.vy < 0.0 && !self.slamming && cc.y > c.center().y + 0.15 {
                    stomp = true;
                }
            }
        }
        for i in kills {
            self.kill(i, true);
        }
        if stomp {
            self.vy = 9.0;
            self.jumps_left = 1;
            self.glide_left = true;
            self.gliding = false;
        }
    }

    /// Plusieurs d'un coup : on compte, et a la fin de la rafale, bonus.
    fn note_multi(&mut self) {
        self.multi_n = if self.multi_t > 0.0 { self.multi_n + 1 } else { 1 };
        self.multi_t = 0.3;
    }

    fn end_multi(&mut self) {
        let n = self.multi_n;
        self.multi_n = 0;
        if n < 3 || self.phase != Phase::Running {
            return;
        }
        let bonus = (n * n * 10) as f32 * (1.0 + 0.15 * self.fat);
        self.score += bonus;
        let (cc, _) = self.hit_circle();
        self.pop(cc + vec2(0.0, 2.2), &format!("MULTI x{n} +{}", bonus as u32), Color::new(1.0, 0.5, 0.9, 1.0));
        let sub = match n {
            3..=4 => "Tarpin bien !",
            5..=7 => if self.gore { "C'est tarpin gore !" } else { "C'est tarpin fort !" },
            8..=11 => "TARPIN DE CARNAGE !",
            _ => "Allez les Girondins, TARPIN !",
        };
        self.say(&format!("{n} D'UN COUP !"), sub);
    }

    fn collide_pickups(&mut self) {
        let (cc, cr) = self.hit_circle();
        let mut got = Vec::new();
        for (i, p) in self.pickups.iter().enumerate() {
            if !p.taken && p.pos.distance(cc) < cr + 0.3 {
                got.push(i);
            }
        }
        for i in got {
            self.pickups[i].taken = true;
            match self.pickups[i].kind {
                PickKind::Kebab => {
                    self.kebab = KEBAB_TIME;
                    self.closeup = CLOSEUP_TIME;
                    self.closeup_len = CLOSEUP_TIME;
                    self.closeup_kind = Closeup::Kebab;
                    self.score += 300.0;
                    self.fat = (self.fat + 0.5).min(MAX_FAT);
                    self.slamming = false;
                }
            }
        }
    }

    /// On laisse le temps de voir la blague avant d'accepter le clic.
    fn closeup_can_skip(&self) -> bool {
        let t = self.closeup_len - self.closeup;
        t > match self.closeup_kind {
            Closeup::Kebab => 1.2,
            Closeup::Litter => 2.0,
        }
    }

    fn end_closeup(&mut self) {
        self.shake(0.3);
        let (cc, _) = self.hit_circle();
        self.dust(self.pos, 20);
        match self.closeup_kind {
            Closeup::Kebab => {
                self.say("KEBAB TURBO !", "Super vitesse, indestructible : fonce !");
                self.pop(cc + vec2(0.0, 1.8), "VROOOUM !", Color::new(1.0, 0.4, 0.1, 1.0));
            }
            Closeup::Litter => {
                self.gave = GAVE_TIME;
                self.gave_count += 1;
                self.score += 500.0;
                self.pop(cc + vec2(0.0, 2.2), "MODE GAVÉ ! +500", Color::new(1.0, 0.85, 0.2, 1.0));
                if self.first_gave {
                    self.first_gave = false;
                    self.say("MODE GAVÉ !", "Tarpin ? A Bordeaux on dit GAVÉ. Rien ne t'arrête !");
                } else {
                    self.say("MODE GAVÉ !", "Gavé fort, même. Fonce dans le tas !");
                }
            }
        }
    }

    fn hurt(&mut self, line: &str) {
        if self.invuln > 0.0 || self.mighty() {
            return;
        }
        self.lives = self.lives.saturating_sub(1);
        self.invuln = 1.6;
        self.hurt_t = 0.6;
        self.fat = (self.fat - 1.0).max(0.0);
        self.combo = 0;
        self.gliding = false;
        self.shake(0.25);
        let (cc, _) = self.hit_circle();
        self.pop(cc + vec2(0.0, 1.0), pick(&["MAOUU !", "AÏE !", "FCHHH !"]), Color::new(1.0, 0.5, 0.3, 1.0));
        for _ in 0..16 {
            self.particles.push(Particle {
                kind: PK::Fur,
                pos: cc + vec2(gen_range(-0.3, 0.3), gen_range(-0.3, 0.3)),
                vel: vec2(gen_range(-3.0, 3.0), gen_range(1.0, 4.0)),
                ang: gen_range(0.0, TAU),
                spin: gen_range(-5.0, 5.0),
                size: gen_range(0.06, 0.12),
                col: if gen_range(0.0, 1.0) < 0.5 { GINGER } else { CREAM },
                life: 2.0,
                max_life: 2.0,
            });
        }
        if self.lives == 0 {
            self.die(Death::KO);
        } else {
            self.say(line, &format!("Plus que {} vie{}", self.lives, if self.lives > 1 { "s" } else { "" }));
            if !self.on_ground {
                self.vy = self.vy.max(4.0);
            }
        }
    }

    fn die(&mut self, how: Death) {
        self.phase = Phase::Over;
        self.death = Some(how);
        self.over_t = 0.0;
        self.slamming = false;
        self.gliding = false;
        self.shake(0.6);
        let (cc, _) = self.hit_circle();
        match how {
            Death::Tram => {
                self.pos.y = 0.0;
                self.pop(cc + vec2(0.0, 1.0), "SCHPLAF !", WHITE);
                self.burst(cc, vec2(-6.0, 1.0), 1.0, GINGER, None);
                if self.gore {
                    // le sang gicle jusque sur l'ecran
                    for _ in 0..3 {
                        self.add_splat();
                    }
                }
                let title = format!("Aplati par le {}", self.tram_name());
                self.say(&title, "Acaboulette est tarpin plat. Comme une crêpe.");
            }
            Death::KO => {
                self.say("Acaboulette est KO", "Tarpin fatigué, le gros matou...");
            }
            Death::Mine => {
                if self.gore {
                    self.explode_cat();
                } else {
                    self.dust(self.pos, 30);
                }
                self.say("Une mine sur les quais !", "Acaboulette en kit. Tarpin de morceaux.");
            }
        }
        let t = self.total();
        if t > self.best {
            self.best = t;
            self.new_record = true;
            save_record(t);
        }
    }

    /// Une bestiole se fait ecrabouiller.
    fn kill(&mut self, i: usize, by_cat: bool) {
        if self.critters[i].dead {
            return;
        }
        self.critters[i].dead = true;
        let (kind, center) = (self.critters[i].kind, self.critters[i].center());
        let ki = kind as usize;
        self.kills[ki] += 1;
        if kind == Kind::Crs {
            self.knock_crs(i, by_cat);
            return;
        }
        // Avale en courant, ou ecrabouille en tombant dessus
        let in_tank = self.tank > 0.0;
        let eaten = by_cat && self.on_ground && !self.slamming && !in_tank;

        if by_cat {
            self.combo += 1;
            self.combo_t = 1.8;
            self.note_multi();
            let mult = (1.0 + 0.5 * (self.combo - 1) as f32).min(6.0) * (1.0 + 0.15 * self.fat);
            let pts = kind.points() * mult;
            self.score += pts;
            self.bloody = (self.bloody + 0.15).min(1.0);
            if !in_tank {
                self.fat = (self.fat + kind.fat_gain() * if eaten { 1.0 } else { 0.6 }).min(MAX_FAT);
            }
            let word = if eaten {
                pick(&["MIAM !", "GLOUPS !", "SLURP !", "CRONCH !", "MIAOUM !"])
            } else {
                pick(&["SPLOTCH !", "SCRONCH !", "SPLAF !", "PAF !"])
            };
            self.pop(center + vec2(0.0, 0.7), &format!("{word} +{}", pts as u32), Color::new(1.0, 0.92, 0.3, 1.0));
            match self.combo {
                3 => {
                    self.say_tarpin("COMBO x3", "Tarpin bien !");
                }
                5 => {
                    let sub = if self.gore { "C'est tarpin gore !" } else { "C'est tarpin fort !" };
                    self.say_tarpin("COMBO x5", sub);
                }
                8 => self.say("COMBO x8 !", "TARPIN DE CARNAGE !"),
                12 => self.say("COMBO x12 !!!", "Allez les Girondins, TARPIN !"),
                20 => {
                    let town = if self.city() == City::Algerie { "Alger" } else { self.city().name() };
                    self.say("COMBO x20 !!!", &format!("{town} n'a plus de pigeons. Tarpin."));
                }
                30 => {
                    let town = if self.city() == City::Algerie { "Alger" } else { self.city().name() };
                    self.say("COMBO x30 !!!", &format!("La SPA de {town} porte plainte"));
                }
                _ => {}
            }
            if kind == Kind::Dog && self.kills[ki] == 1 {
                self.say_tarpin("Un chien avalé tout rond !", "Le teckel de la rue Notre-Dame. Tarpin croustillant.");
            } else if kind == Kind::Gull && self.kills[ki] == 1 && !in_tank {
                self.say_tarpin("Mouette gobée en plein vol !", "Un petit goût d'iode et de frites volées. Tarpin la classe.");
            } else if kind == Kind::Rat && self.kills[ki] == 1 {
                match self.city() {
                    City::Bordeaux => self.say_tarpin("Rat de la Garonne", "Ça a un petit goût de vase. Tarpin bon quand même."),
                    City::Marseille => self.say_tarpin("Rat du Vieux-Port", "Nourri à la bouillabaisse. Tarpin bon."),
                    City::Forcalquier => self.say_tarpin("Rat des champs", "Élevé à la lavande et au banon. Tarpin fin."),
                    City::Algerie => self.say_tarpin("Rat du port d'Alger", "Un petit goût de sardine grillée. Tarpin bon."),
                };
            }
        } else {
            self.score += kind.points() * 0.5;
            self.pop(center + vec2(0.0, 0.7), "SCRITCH !", Color::new(0.9, 0.9, 1.0, 1.0));
        }

        let dir = if by_cat { vec2(self.speed * 0.6, 2.5) } else { vec2(-4.0, 2.0) };
        let feathers = match kind {
            Kind::Pigeon => Some(Color::new(0.62, 0.64, 0.7, 1.0)),
            Kind::Gull => Some(WHITE),
            _ => None,
        };
        let amount = match kind {
            Kind::Dog => 1.4,
            _ if eaten => 0.5,
            _ => 1.0,
        };
        self.burst(center, dir, amount, kind.color(), feathers);
    }

    /// Les CRS ne saignent pas : ils valdinguent, casque et matraque en l'air.
    fn knock_crs(&mut self, i: usize, by_cat: bool) {
        let c = &self.critters[i];
        let (pos, phase) = (c.pos, c.phase);
        let center = c.center();
        let slam = self.slamming || self.on_ground;
        let vel = if by_cat {
            if slam { vec2(gen_range(4.0, 9.0), gen_range(9.0, 13.0)) } else { vec2(self.speed * 0.4 + 2.0, gen_range(5.0, 8.0)) }
        } else {
            vec2(-9.0, 7.0)
        };
        self.flung.push(Flung { pos, vel, ang: 0.0, spin: gen_range(-14.0, -7.0), phase });
        let navy = Kind::Crs.color();
        for (col, size, v) in [
            (navy, 0.16, vec2(gen_range(-2.0, 2.0), gen_range(6.0, 9.0))),
            (INK, 0.25, vec2(gen_range(-3.0, 3.0), gen_range(5.0, 8.0))),
        ] {
            self.particles.push(Particle {
                kind: PK::Debris,
                pos: center + vec2(0.0, 0.6),
                vel: v,
                ang: gen_range(0.0, TAU),
                spin: gen_range(-15.0, 15.0),
                size,
                col,
                life: 6.0,
                max_life: 6.0,
            });
        }
        for _ in 0..10 {
            let a = gen_range(0.0, TAU);
            self.particles.push(Particle {
                kind: PK::Star,
                pos: center + vec2(0.0, 0.5),
                vel: vec2(a.cos(), a.sin()) * gen_range(2.0, 4.0),
                ang: a,
                spin: gen_range(-6.0, 6.0),
                size: gen_range(0.07, 0.13),
                col: [YELLOW, WHITE][gen_range(0, 2)],
                life: 0.8,
                max_life: 0.8,
            });
        }
        if by_cat {
            self.combo += 1;
            self.combo_t = 1.8;
            self.note_multi();
            let mult = (1.0 + 0.5 * (self.combo - 1) as f32).min(6.0) * (1.0 + 0.15 * self.fat);
            let pts = Kind::Crs.points() * mult;
            self.score += pts;
            self.pop(
                center + vec2(0.0, 1.0),
                &format!("{} +{}", pick(&["BONK !", "DONG !", "POING !", "VLAN !"]), pts as u32),
                Color::new(0.6, 0.85, 1.0, 1.0),
            );
            let n = self.kills[Kind::Crs as usize];
            if n == 1 {
                self.say_tarpin("CRS en orbite !", "Il a pas vu venir le matou. Tarpin de vol plané.");
            } else if n.is_multiple_of(10) {
                self.say(&format!("{n} CRS dégommés !"), "La préfecture est tarpin énervée");
            }
        } else {
            self.score += Kind::Crs.points() * 0.5;
            self.pop(center + vec2(0.0, 1.0), "BONK !", Color::new(0.9, 0.9, 1.0, 1.0));
        }
        self.shake(0.12);
    }

    fn burst(&mut self, at: Vec2, dir: Vec2, amount: f32, body: Color, feathers: Option<Color>) {
        if self.particles.len() > 2500 {
            self.particles.drain(0..500);
        }
        if self.gore {
            let n = (40.0 * amount) as usize;
            for _ in 0..n {
                let a = gen_range(0.0, TAU);
                let sp = gen_range(1.0, 7.0);
                self.particles.push(Particle {
                    kind: PK::Blood,
                    pos: at,
                    vel: dir * gen_range(0.2, 1.0) + vec2(a.cos(), a.sin().abs() * 1.3) * sp,
                    ang: 0.0,
                    spin: 0.0,
                    size: gen_range(0.025, 0.07),
                    col: blood(gen_range(0.0, 1.0)),
                    life: 3.0,
                    max_life: 3.0,
                });
            }
            let m = (6.0 * amount) as usize;
            for j in 0..m {
                let col = match j % 3 {
                    0 => body,
                    1 => Color::new(0.7, 0.12, 0.15, 1.0),
                    _ => if j % 2 == 0 { BONE } else { Color::new(0.85, 0.4, 0.45, 1.0) },
                };
                self.particles.push(Particle {
                    kind: PK::Chunk,
                    pos: at,
                    vel: dir * gen_range(0.3, 1.1) + vec2(gen_range(-3.0, 3.0), gen_range(2.0, 7.0)),
                    ang: gen_range(0.0, TAU),
                    spin: gen_range(-14.0, 14.0),
                    size: gen_range(0.06, 0.13),
                    col,
                    life: 7.0,
                    max_life: 7.0,
                });
            }
        } else {
            for _ in 0..12 {
                let a = gen_range(0.0, TAU);
                self.particles.push(Particle {
                    kind: PK::Star,
                    pos: at,
                    vel: vec2(a.cos(), a.sin()) * gen_range(2.0, 5.0),
                    ang: a,
                    spin: gen_range(-6.0, 6.0),
                    size: gen_range(0.06, 0.12),
                    col: [YELLOW, PINK, SKYBLUE, WHITE][gen_range(0, 4)],
                    life: 0.8,
                    max_life: 0.8,
                });
            }
        }
        if let Some(fc) = feathers {
            for _ in 0..14 {
                self.particles.push(Particle {
                    kind: PK::Feather,
                    pos: at + vec2(gen_range(-0.15, 0.15), gen_range(-0.1, 0.1)),
                    vel: dir * gen_range(0.1, 0.4) + vec2(gen_range(-2.5, 2.5), gen_range(1.0, 4.0)),
                    ang: gen_range(0.0, TAU),
                    spin: gen_range(-3.0, 3.0),
                    size: gen_range(0.08, 0.14),
                    col: mix(fc, WHITE, gen_range(0.0, 0.4)),
                    life: 3.5,
                    max_life: 3.5,
                });
            }
        }
    }

    fn add_splat(&mut self) {
        let w = screen_width();
        let h = screen_height();
        let r = gen_range(26.0, 70.0);
        let mut sats = [(Vec2::ZERO, 0.0); 5];
        for s in &mut sats {
            let a = gen_range(0.0, TAU);
            *s = (Vec2::from_angle(a) * r * gen_range(0.9, 1.7), r * gen_range(0.15, 0.4));
        }
        self.splats.push(Splat {
            pos: vec2(gen_range(w * 0.1, w * 0.9), gen_range(h * 0.1, h * 0.7)),
            r,
            life: 4.0,
            max_life: 4.0,
            drip: 0.0,
            drip_speed: gen_range(10.0, 30.0),
            sats,
        });
    }

    fn update_particles(&mut self, dt: f32) {
        let mut new_decals = Vec::new();
        for p in &mut self.particles {
            p.life -= dt;
            p.ang += p.spin * dt;
            match p.kind {
                PK::Blood | PK::Water => {
                    p.vel.y -= 20.0 * dt;
                    p.pos += p.vel * dt;
                    if p.pos.y <= 0.0 {
                        if p.kind == PK::Blood {
                            new_decals.push(Decal { x: p.pos.x, w: p.size * gen_range(2.0, 5.0), shade: gen_range(0.0, 1.0) });
                        }
                        p.life = 0.0;
                    }
                }
                PK::Chunk | PK::Debris | PK::Eye => {
                    p.vel.y -= 20.0 * dt;
                    p.pos += p.vel * dt;
                    if p.pos.y <= p.size * 0.5 {
                        p.pos.y = p.size * 0.5;
                        if p.vel.y < -1.0 {
                            p.vel.y = -p.vel.y * 0.3;
                        } else {
                            p.vel.y = 0.0;
                        }
                        p.vel.x *= 0.6;
                        p.spin *= 0.6;
                    }
                }
                PK::Feather | PK::Fur => {
                    p.vel.y -= 4.0 * dt;
                    p.vel *= 1.0 - 2.5 * dt;
                    p.vel.x += (p.life * 5.0).sin() * dt * 4.0;
                    p.pos += p.vel * dt;
                    if p.pos.y < 0.03 {
                        p.pos.y = 0.03;
                        p.vel = Vec2::ZERO;
                    }
                }
                PK::Star => {
                    p.vel *= 1.0 - 3.0 * dt;
                    p.pos += p.vel * dt;
                }
                PK::Dust => {
                    p.vel *= 1.0 - 4.0 * dt;
                    p.pos += p.vel * dt;
                    p.size += dt * 0.5;
                }
            }
        }
        for f in &mut self.flung {
            f.vel.y -= 20.0 * dt;
            f.pos += f.vel * dt;
            f.ang += f.spin * dt;
            f.phase += dt * 3.0;
        }
        let back = self.pos.x - 16.0;
        self.flung.retain(|f| f.pos.y > -6.0 && f.pos.x > back);
        self.particles.retain(|p| p.life > 0.0 && p.pos.x > back);
        if self.gore {
            self.decals.extend(new_decals);
            if self.decals.len() > 600 {
                self.decals.drain(0..100);
            }
        }
        for s in &mut self.splats {
            s.life -= dt;
            s.drip += s.drip_speed * dt;
        }
        self.splats.retain(|s| s.life > 0.0);
        for p in &mut self.pops {
            p.t += dt;
        }
        self.pops.retain(|p| p.t < 1.0);
    }

    // -----------------------------------------------------------------------
    // Generation du parcours
    // -----------------------------------------------------------------------

    fn spawn_pattern(&mut self) {
        let x = self.next_spawn;
        let d = (x / 1500.0).min(1.0); // difficulte
        let gap = self.speed * gen_range(0.8, 1.4);
        let can_obstacle = x > self.no_obstacle_until;
        let roll = gen_range(0.0, 1.0);

        // Un char d'assaut a l'arret, de temps en temps
        if self.tank == 0.0
            && self.tank_spots.is_empty()
            && x > self.next_tank_x
            && can_obstacle
            && gen_range(0.0, 1.0) < TANK_CHANCE
        {
            self.tank_spots.push(TankSpot { x: x + 5.0 });
            self.no_obstacle_until = x + 16.0;
            self.next_tank_x = x + TANK_EVERY;
            self.next_spawn = x + 14.0;
            return;
        }

        // Le tram B, de temps en temps
        if x > 140.0 && can_obstacle && roll < 0.06 + 0.06 * d {
            self.trams.push(Tram { x: x + 26.0, warned: false, rode: false });
            self.no_obstacle_until = x + 60.0;
            for i in 0..gen_range(2, 5) {
                self.spawn_critter(Kind::Pigeon, x + 8.0 + i as f32 * 1.2);
            }
            self.next_spawn = x + 40.0;
            return;
        }

        let roll = gen_range(0.0, 1.0);
        if roll < 0.32 {
            // Une bande de pigeons
            let n = gen_range(3, 8);
            for i in 0..n {
                self.spawn_critter(Kind::Pigeon, x + i as f32 * gen_range(0.6, 1.0));
            }
        } else if roll < 0.42 {
            for i in 0..gen_range(2, 5) {
                self.spawn_critter(Kind::Rat, x + i as f32 * gen_range(0.8, 1.6));
            }
        } else if roll < 0.52 {
            for i in 0..gen_range(1, 3) {
                self.spawn_critter(Kind::Dog, x + i as f32 * 2.5);
            }
        } else if roll < 0.57 && x > 120.0 {
            // Barrage de CRS : le gros bonus
            for i in 0..gen_range(3, 7) {
                self.spawn_critter(Kind::Crs, x + 2.0 + i as f32 * 0.9);
            }
        } else if roll < 0.68 && can_obstacle && x > 60.0 {
            // Voiture ou moto en face
            let kind = if gen_range(0.0, 1.0) < 0.6 { VehKind::Car } else { VehKind::Moto };
            let speed = match kind {
                VehKind::Car => gen_range(3.0, 5.0 + 3.0 * d),
                VehKind::Moto => gen_range(7.0, 9.0 + 3.0 * d),
            };
            let colors = [
                Color::new(0.85, 0.15, 0.15, 1.0),
                Color::new(0.2, 0.4, 0.8, 1.0),
                Color::new(0.95, 0.95, 0.92, 1.0),
                Color::new(0.2, 0.2, 0.22, 1.0),
                Color::new(0.95, 0.75, 0.2, 1.0),
                BORDEAUX,
            ];
            self.vehicles.push(Vehicle { kind, x: x + 14.0, speed, color: colors[gen_range(0, colors.len())], hit: false, honked: false });
            self.no_obstacle_until = x + 26.0;
            for i in 0..gen_range(0, 4) {
                self.spawn_critter(Kind::Pigeon, x + 2.0 + i as f32 * 0.8);
            }
        } else if roll < 0.84 && can_obstacle {
            let kind = [ObsKind::Barrier, ObsKind::Bike, ObsKind::Bin][gen_range(0, 3)];
            let (w, h) = match kind {
                ObsKind::Barrier => (1.4, 1.0),
                ObsKind::Bike => (1.7, 1.0),
                ObsKind::Bin => (0.75, 1.15),
            };
            self.obstacles.push(Obstacle { kind, x, w, h, knocked: 0.0, smashed: false });
            // un pigeon perche dessus, ou des pigeons derriere
            if gen_range(0.0, 1.0) < 0.5 {
                for i in 0..gen_range(2, 5) {
                    self.spawn_critter(Kind::Pigeon, x + w + 1.5 + i as f32 * 0.8);
                }
            }
            if gen_range(0.0, 1.0) < 0.35 {
                self.spawn_critter(Kind::Rat, x + w + 3.0);
            }
        } else if roll < 0.95 && can_obstacle && x > 50.0 {
            // Mines, parfois deux de suite, souvent au milieu des pigeons
            if gen_range(0.0, 1.0) < 0.35 + 0.4 * d {
                let n = if d > 0.4 && gen_range(0.0, 1.0) < 0.3 { 2 } else { 1 };
                for i in 0..n {
                    self.hazards.push(Hazard { kind: HazKind::Mine, x: x + i as f32 * gen_range(3.5, 5.0), fired: false, t: 0.0 });
                }
            }
            if gen_range(0.0, 1.0) < 0.6 {
                for i in 0..gen_range(2, 5) {
                    self.spawn_critter(Kind::Pigeon, x - 2.5 + i as f32 * 0.7);
                }
            }
        } else {
            for i in 0..gen_range(4, 9) {
                self.spawn_critter(Kind::Pigeon, x + i as f32 * 0.7);
            }
        }

        // Des mouettes, tout la-haut : il faut le deltaplane
        if x > 60.0 && gen_range(0.0, 1.0) < 0.14 {
            let y = gen_range(6.4, 7.4);
            for i in 0..gen_range(1, 4) {
                self.spawn_critter(Kind::Gull, x + 6.0 + i as f32 * gen_range(1.6, 2.4));
                if let Some(g) = self.critters.last_mut() {
                    g.pos.y = y + gen_range(-0.4, 0.4);
                    g.base_y = g.pos.y;
                }
            }
        }

        if !self.mighty() && x > 250.0 && gen_range(0.0, 1.0) < 0.03 {
            self.pickups.push(Pickup { kind: PickKind::Kebab, pos: vec2(x + gap * 0.3, 1.5), taken: false, phase: 0.0 });
        }
        self.next_spawn = x + gap;
    }

    fn spawn_critter(&mut self, kind: Kind, x: f32) {
        let (vx, y) = match kind {
            Kind::Pigeon => (gen_range(-0.7, -0.2), 0.0),
            Kind::Rat => (gen_range(-3.5, -2.0), 0.0),
            Kind::Dog => (gen_range(-3.0, -0.8), 0.0),
            Kind::Crs => (gen_range(-0.6, -0.2), 0.0),
            Kind::Gull => (gen_range(-1.6, -0.8), 7.0),
        };
        self.critters.push(Critter {
            kind,
            pos: vec2(x, y),
            vel: vec2(vx, 0.0),
            phase: gen_range(0.0, TAU),
            base_y: if kind == Kind::Dog { gen_range(0.0, 1.0) } else { y },
            flying: false,
            flag: false,
            dead: false,
        });
    }

    // -----------------------------------------------------------------------
    // Dessin
    // -----------------------------------------------------------------------

    fn view(&self) -> View {
        let h = screen_height();
        let w = screen_width();
        let mut shake = Vec2::ZERO;
        if self.shake_t > 0.0 {
            let a = self.shake_t * 30.0;
            shake = vec2(gen_range(-a, a), gen_range(-a, a));
        }
        View { cam_x: self.pos.x, cam_y: self.cam_y, s: h / 10.0, ground_sy: h * 0.8, anchor_x: w * 0.22, shake }
    }

    /// 0 = plein jour, 1 = nuit noire ; et intensite du coucher de soleil.
    fn daylight(&self) -> (f32, f32) {
        let t = (self.pos.x.max(0.0) / 1800.0).fract();
        let night = if t < 0.45 {
            0.0
        } else if t < 0.6 {
            (t - 0.45) / 0.15
        } else if t < 0.88 {
            1.0
        } else {
            1.0 - (t - 0.88) / 0.12
        };
        let sunset = (1.0 - ((t - 0.5) / 0.09).abs()).max(0.0) + (1.0 - ((t - 0.92) / 0.06).abs()).max(0.0) * 0.6;
        (night, sunset.min(1.0))
    }

    fn draw(&self) {
        let v = self.view();
        let (night, sunset) = self.daylight();
        self.draw_sky(&v, night, sunset);
        self.draw_background(&v, night);
        self.draw_ground(&v, night);
        self.draw_decals(&v);
        for o in &self.obstacles {
            draw_obstacle(&v, o);
        }
        for hz in &self.hazards {
            draw_hazard(&v, hz, self.time);
        }
        for p in &self.pickups {
            draw_pickup(&v, p, self.time);
        }
        for t in &self.trams {
            draw_tram(&v, t, night, self.time, zone_at(t.x).city());
        }
        for ve in &self.vehicles {
            draw_vehicle(&v, ve, night, self.time);
        }
        for t in &self.tank_spots {
            draw_tank(&v, t.x, 0.0, 0.0, None);
            draw_tank_arrow(&v, t.x, self.time);
        }
        for c in &self.critters {
            if !c.dead {
                draw_critter(&v, c, self.time);
            }
        }
        for f in &self.flung {
            draw_crs(&v, f.pos, f.ang, f.phase, false);
        }
        self.draw_particles(&v, true);
        self.draw_cat(&v);
        for sh in &self.shells {
            draw_shell(&v, sh.pos, self.time);
        }
        for m in &self.missiles {
            draw_missile(&v, m, self.time);
        }
        self.draw_particles(&v, false);
        self.draw_weather(&v);
        self.draw_pops(&v);
        if night > 0.0 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.02, 0.03, 0.12, night * 0.18));
        }
        self.draw_splats();
        self.draw_hud();
        if self.closeup > 0.0 {
            self.draw_closeup();
        }
    }

    /// Pause gros plan : la tete d'Acaboulette qui tire la langue, et le kebab.
    fn draw_closeup(&self) {
        match self.closeup_kind {
            Closeup::Kebab => self.draw_closeup_kebab(),
            Closeup::Litter => self.draw_closeup_litter(),
        }
    }

    fn draw_closeup_kebab(&self) {
        let w = screen_width();
        let h = screen_height();
        let t = self.closeup_len - self.closeup; // temps ecoule
        let time = self.time;

        comic_rays(Color::new(0.75, 0.2, 0.05, 1.0), Color::new(1.0, 0.65, 0.1, 1.0), time);
        // zoom avec rebond
        let z = (t / 0.25).min(1.0);
        let z = 1.0 - (1.0 - z) * (1.0 - z);
        let z = z * (1.0 + 0.04 * (time * 9.0).sin());
        let unit = h * 0.27 * z;
        let shake = if t < 0.3 { vec2(gen_range(-6.0, 6.0), gen_range(-6.0, 6.0)) } else { Vec2::ZERO };

        // La tete, de face
        let hv = View { cam_x: 0.0, cam_y: 0.0, s: unit, ground_sy: h * 0.58 + (time * 6.0).sin() * 6.0, anchor_x: w * 0.36, shake };
        let v = &hv;
        for sx in [-1.0f32, 1.0] {
            tri(v, vec2(0.9 * sx, 0.35), vec2(0.25 * sx, 0.85), vec2(0.82 * sx, 1.35), GINGER);
            tri(v, vec2(0.78 * sx, 0.5), vec2(0.38 * sx, 0.82), vec2(0.75 * sx, 1.15), NOSE);
        }
        ellipse(v, vec2(0.0, 0.0), 1.15, 0.95, 0.0, GINGER);
        for sx in [-1.0f32, 1.0] {
            ellipse(v, vec2(0.62 * sx, -0.3), 0.55, 0.42, 0.0, GINGER); // grosses joues
            for k in 0..3 {
                let y = -0.05 - k as f32 * 0.18;
                seg(v, vec2(1.12 * sx, y), vec2(0.88 * sx, y - 0.03), 0.07, GINGER_DARK);
            }
        }
        for k in 0..3 {
            let x = -0.22 + k as f32 * 0.22;
            seg(v, vec2(x, 0.88), vec2(x * 0.8, 0.55), 0.08, GINGER_DARK); // le M
        }
        // yeux exorbites qui louchent
        let looks = [vec2(0.07, 0.07), vec2(-0.07, -0.05)];
        for (k, sx) in [-1.0f32, 1.0].into_iter().enumerate() {
            let e = vec2(0.42 * sx, 0.2);
            let big = if k == 0 { 1.0 } else { 0.8 };
            ellipse(v, e, 0.27 * big, 0.3 * big, 0.0, WHITE);
            ellipse(v, e + looks[k], 0.17 * big, 0.2 * big, 0.0, Color::new(0.45, 0.75, 0.2, 1.0));
            circ(v, e + looks[k], 0.12 * big, INK);
            circ(v, e + looks[k] + vec2(0.05, 0.06), 0.04, WHITE);
        }
        // museau, nez
        for sx in [-1.0f32, 1.0] {
            ellipse(v, vec2(0.22 * sx, -0.34), 0.34, 0.26, 0.0, CREAM);
        }
        ellipse(v, vec2(0.0, -0.55), 0.3, 0.18, 0.0, CREAM);
        tri(v, vec2(-0.12, -0.12), vec2(0.12, -0.12), vec2(0.0, -0.25), NOSE);
        // bouche grande ouverte et la langue qui pendouille
        ellipse(v, vec2(0.0, -0.52), 0.26, 0.17, 0.0, Color::new(0.4, 0.05, 0.08, 1.0));
        let wag = (time * 11.0).sin() * 0.12;
        let tip = vec2(0.12 + wag, -1.15);
        seg(v, vec2(0.0, -0.52), vec2(0.06 + wag * 0.5, -0.85), 0.3, NOSE);
        seg(v, vec2(0.06 + wag * 0.5, -0.85), tip, 0.26, NOSE);
        line(v, vec2(0.0, -0.58), tip + vec2(0.0, 0.1), 0.025, Color::new(0.8, 0.35, 0.4, 1.0));
        // sauce blanche au coin de la bouche
        ellipse(v, vec2(0.3, -0.5), 0.09, 0.06, 0.0, WHITE);
        let drip = (t * 0.25).min(0.25);
        seg(v, vec2(0.32, -0.52), vec2(0.32, -0.55 - drip), 0.05, WHITE);
        // moustaches
        for sx in [-1.0f32, 1.0] {
            for k in 0..3 {
                let y = -0.3 - k as f32 * 0.1;
                line(v, vec2(0.45 * sx, y), vec2(1.55 * sx, y + 0.15 - k as f32 * 0.15), 0.018, WHITE);
            }
        }

        // Le kebab, qui flotte et fume
        let kv = View {
            cam_x: 0.0,
            cam_y: 0.0,
            s: unit,
            ground_sy: h * 0.62 + (time * 4.0).sin() * 10.0,
            anchor_x: w * 0.77,
            shake,
        };
        for k in 0..3 {
            let x = -0.35 + k as f32 * 0.35;
            let ph = time * 3.0 + k as f32;
            let a = 0.4 + 0.2 * ph.sin();
            for j in 0..4 {
                let y0 = 0.95 + j as f32 * 0.15;
                let off = (ph + j as f32).sin() * 0.08;
                line(&kv, vec2(x + off, y0), vec2(x - off, y0 + 0.18), 0.04, Color::new(1.0, 1.0, 1.0, a));
            }
        }
        draw_kebab(&kv, vec2(0.0, 0.0), 1.9);

        // Le texte
        let txt = "Un kebab ! Je deviens tarpin puissant !!";
        let mut size = 56u16;
        while size > 20 && measure_text(txt, None, size, 1.0).width > w - 60.0 {
            size -= 2;
        }
        let pop = 1.0 + 0.3 * (1.0 - (t / 0.2).min(1.0));
        comic_text(txt, w / 2.0, h * 0.12, (size as f32 * pop) as u16, Color::new(1.0, 0.95, 0.3, 1.0), Color::new(0.35, 0.02, 0.02, 1.0));
        comic_text(
            "Super vitesse et indestructible !",
            w / 2.0,
            h * 0.12 + 56.0,
            (size as f32 * 0.62) as u16,
            WHITE,
            Color::new(0.35, 0.02, 0.02, 1.0),
        );
        self.draw_continue_hint();
    }

    fn draw_continue_hint(&self) {
        if !self.closeup_can_skip() {
            return;
        }
        let a = 0.65 + 0.35 * (self.time * 5.0).sin();
        comic_text(
            "Clic / Espace pour continuer",
            screen_width() / 2.0,
            screen_height() * 0.955,
            30,
            Color::new(1.0, 1.0, 1.0, a),
            Color::new(0.1, 0.0, 0.1, a),
        );
    }

    /// Pause litiere : Acaboulette, trop gros, pousse sa crotte en lisant le journal.
    fn draw_closeup_litter(&self) {
        let w = screen_width();
        let h = screen_height();
        let t = self.closeup_len - self.closeup;
        let time = self.time;
        let done = t > 1.5; // la crotte est sortie

        comic_rays(Color::new(0.3, 0.12, 0.4, 1.0), Color::new(0.45, 0.25, 0.6, 1.0), time);
        let z = (t / 0.25).min(1.0);
        let z = 1.0 - (1.0 - z) * (1.0 - z);
        let unit = h * 0.165 * z;
        let strain = if done { Vec2::ZERO } else { vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)) * (2.0 + t * 4.0) };
        let lv = View { cam_x: 0.0, cam_y: 0.0, s: unit, ground_sy: h * 0.88, anchor_x: w * 0.5, shake: strain };
        let v = &lv;

        // le bac a litiere (arriere)
        let tray = Color::new(0.2, 0.55, 0.85, 1.0);
        let tray_d = Color::new(0.12, 0.38, 0.65, 1.0);
        ellipse(v, vec2(-0.3, 0.9), 3.35, 0.3, 0.0, tray_d);
        ellipse(v, vec2(-0.3, 0.9), 3.2, 0.22, 0.0, Color::new(0.82, 0.8, 0.74, 1.0));

        // la queue, dressee bien haut
        let mut prev = vec2(-1.25, 1.75);
        for k in 1..=8 {
            let a = PI * 0.62 - k as f32 * 0.1 + (time * 4.0).sin() * 0.08;
            let p = prev + Vec2::from_angle(a) * 0.3;
            seg(v, prev, p, 0.26 - k as f32 * 0.012, if k % 2 == 0 { GINGER_DARK } else { GINGER });
            prev = p;
        }

        // le corps, de profil, accroupi (arriere bas, poitrail haut)
        let bc = vec2(-0.1, 1.85);
        ellipse(v, bc, 1.45, 1.0, 0.32, GINGER);
        for k in 0..5 {
            let x = -0.9 + k as f32 * 0.42;
            let top = bc + Vec2::from_angle(0.32).rotate(vec2(x, 0.95));
            let low = bc + Vec2::from_angle(0.32).rotate(vec2(x - 0.1, 0.45));
            seg(v, top, low, 0.13, GINGER_DARK);
        }
        ellipse(v, bc + vec2(0.35, -0.35), 1.0, 0.55, 0.32, CREAM);
        // cuisse et patte arriere repliees
        ellipse(v, vec2(-0.75, 1.35), 0.7, 0.55, 0.2, GINGER);
        seg(v, vec2(-0.95, 1.55), vec2(-0.75, 1.05), 0.07, GINGER_DARK);
        ellipse(v, vec2(-0.35, 0.98), 0.45, 0.14, 0.0, CREAM);
        // pattes avant bien droites, cramponnees
        for (dx, col) in [(0.0, GINGER_SHADE), (0.25, GINGER)] {
            seg(v, vec2(0.85 + dx, 2.0), vec2(1.0 + dx, 1.0), 0.26, col);
            ellipse(v, vec2(1.05 + dx, 0.97), 0.2, 0.1, 0.0, CREAM);
        }

        // la crotte, sous la queue : une belle grosse
        let pb = vec2(-2.45, 0.92);
        if done {
            let g = ((t - 1.5) / 0.35).min(1.0);
            let g = g * (1.0 + 0.08 * (1.0 - g)); // petit rebond
            let poo = Color::new(0.4, 0.24, 0.1, 1.0);
            let poo_l = Color::new(0.52, 0.32, 0.14, 1.0);
            let poo_d = Color::new(0.3, 0.17, 0.07, 1.0);
            let layers = [(0.0, 0.78, 0.3), (0.3, 0.64, 0.27), (0.57, 0.5, 0.24), (0.8, 0.36, 0.2), (1.0, 0.22, 0.15)];
            for (k, (y, rx, ry)) in layers.into_iter().enumerate() {
                let c = pb + vec2(0.03 * k as f32, (0.12 + y) * g);
                ellipse(v, c, rx * g, ry * g, 0.0, if k % 2 == 0 { poo } else { poo_l });
                ellipse(v, c + vec2(0.0, -ry * 0.55 * g), rx * 0.9 * g, ry * 0.25 * g, 0.0, poo_d);
                ellipse(v, c + vec2(-rx * 0.4 * g, ry * 0.35 * g), rx * 0.25 * g, ry * 0.18 * g, 0.0, Color::new(0.7, 0.5, 0.3, 0.6));
            }
            let tip = pb + vec2(0.15, 1.25 * g);
            tri(v, tip + vec2(-0.14, -0.08), tip + vec2(0.1, -0.1), tip + vec2(0.22, 0.2) * g, poo_l);
            // odeur et mouches
            for k in 0..4 {
                let x0 = pb.x - 0.5 + k as f32 * 0.33;
                let a = (0.5 + 0.5 * (time * 3.0 + k as f32).sin()) * g;
                let mut pp = vec2(x0, pb.y + 1.6);
                for j in 0..6 {
                    let np = vec2(x0 + (time * 4.0 + j as f32 + k as f32).sin() * 0.1, pp.y + 0.18);
                    line(v, pp, np, 0.05, Color::new(0.45, 0.75, 0.2, a));
                    pp = np;
                }
            }
            for k in 0..3 {
                let a = time * 6.0 + k as f32 * TAU / 3.0;
                let f = pb + vec2(a.cos() * 0.9, 1.6 + (a * 1.3).sin() * 0.35);
                circ(v, f, 0.06, INK);
                ellipse(v, f + vec2(0.0, 0.07), 0.07, 0.035, (time * 40.0).sin(), Color::new(0.9, 0.95, 1.0, 0.8));
            }
        }

        // la tete, de profil (vers la droite)
        let hc = vec2(1.35, 3.0);
        let hp = |x: f32, y: f32| hc + vec2(x, y);
        tri(v, hp(-0.55, 0.45), hp(-0.1, 0.6), hp(-0.45, 1.15), GINGER_DARK);
        tri(v, hp(-0.25, 0.55), hp(0.25, 0.55), hp(0.0, 1.2), GINGER);
        tri(v, hp(-0.12, 0.6), hp(0.15, 0.6), hp(0.0, 1.0), NOSE);
        ellipse(v, hc, 0.8, 0.7, 0.0, GINGER);
        for k in 0..3 {
            let x = -0.35 + k as f32 * 0.18;
            seg(v, hp(x, 0.62), hp(x + 0.05, 0.35), 0.07, GINGER_DARK);
        }
        ellipse(v, hp(0.55, -0.25), 0.42, 0.3, 0.0, CREAM); // museau
        tri(v, hp(0.82, -0.02), hp(0.98, -0.04), hp(0.9, -0.16), NOSE);
        ellipse(v, hp(0.15, -0.2), 0.2, 0.11, 0.0, Color::new(1.0, 0.3, 0.3, if done { 0.35 } else { 0.75 }));
        let e = hp(0.35, 0.15);
        if done {
            // oeil plisse de bonheur et sourire
            seg(v, e + vec2(-0.16, -0.03), e + vec2(0.0, 0.1), 0.06, INK);
            seg(v, e + vec2(0.0, 0.1), e + vec2(0.16, -0.03), 0.06, INK);
            seg(v, hp(0.55, -0.42), hp(0.75, -0.35), 0.04, INK);
            seg(v, hp(0.45, -0.38), hp(0.55, -0.42), 0.04, INK);
        } else {
            // oeil serre et dents serrees
            seg(v, e + vec2(-0.16, 0.1), e + vec2(0.12, 0.0), 0.06, INK);
            seg(v, e + vec2(0.12, 0.0), e + vec2(-0.16, -0.1), 0.06, INK);
            rect(v, hc.x + 0.45, hc.y - 0.5, hc.x + 0.85, hc.y - 0.34, WHITE);
            for k in 0..3 {
                let x = 0.55 + k as f32 * 0.1;
                line(v, hp(x, -0.5), hp(x, -0.34), 0.015, INK);
            }
            line(v, hp(0.45, -0.42), hp(0.85, -0.42), 0.015, INK);
            for k in 0..2 {
                let fall = ((time * 1.5 + k as f32 * 0.5).fract()) * 0.5;
                let d = hp(-0.5 + k as f32 * 0.9, 0.75 - fall);
                circ(v, d, 0.07, Color::new(0.6, 0.85, 1.0, 0.9));
                tri(v, d + vec2(-0.06, 0.02), d + vec2(0.06, 0.02), d + vec2(0.0, 0.16), Color::new(0.6, 0.85, 1.0, 0.9));
            }
        }
        for k in 0..3 {
            let y = -0.2 - k as f32 * 0.08;
            line(v, hp(0.6, y), hp(1.5, y + 0.12 - k as f32 * 0.12), 0.016, WHITE);
        }

        // le devant du bac (cache les pattes et le bas de la crotte)
        let q = [vec2(-3.2, 0.0), vec2(2.6, 0.0), vec2(3.0, 0.9), vec2(-3.6, 0.9)];
        tri(v, q[0], q[1], q[2], tray);
        tri(v, q[0], q[2], q[3], tray);
        rect(v, -3.65, 0.8, 3.05, 0.95, tray_d);

        // les textes
        let (txt, col) = if done {
            ("Ahhh... ça va tarpin mieux ! MODE GAVÉ !", Color::new(1.0, 0.9, 0.3, 1.0))
        } else {
            ("Je suis tarpin gavé... pause litière !", Color::new(1.0, 1.0, 1.0, 1.0))
        };
        let mut size = 54u16;
        while size > 20 && measure_text(txt, None, size, 1.0).width > w - 60.0 {
            size -= 2;
        }
        comic_text(txt, w / 2.0, h * 0.09, size, col, Color::new(0.25, 0.05, 0.3, 1.0));
        if done {
            let a = ((t - 1.5) / 0.15).min(1.0);
            // nuage de prout qui sort du derriere
            for k in 0..5 {
                let age = (t - 1.5 + k as f32 * 0.12).max(0.0);
                let c = vec2(-1.55 - age * 0.9 - k as f32 * 0.15, 1.5 + (age * 3.0 + k as f32).sin() * 0.15 + k as f32 * 0.08);
                let r = 0.18 + age * 0.25;
                let fade = (1.0 - age / 2.5).max(0.0) * a;
                circ(v, c, r, Color::new(0.55, 0.75, 0.25, 0.45 * fade));
            }
            let pp = v.p(vec2(-3.5, 3.0));
            let wob = (time * 10.0).sin() * 0.06;
            comic_text(
                "PROUT !",
                pp.x,
                pp.y,
                ((unit * 0.55) * (1.0 + wob)) as u16,
                Color::new(0.6, 0.85, 0.3, a),
                Color::new(0.1, 0.25, 0.05, a),
            );
        } else {
            let pp = v.p(vec2(-2.6, 3.4));
            let s2 = 30 + ((time * 8.0).sin() * 4.0) as i32;
            comic_text("NNNGH !", pp.x, pp.y, s2 as u16, WHITE, Color::new(0.4, 0.0, 0.0, 1.0));
        }
        if done {
            centered("Trop gros : invincible, il fonce dans le tas !", h * 0.06 + 70.0, 24, WHITE);
        }
        self.draw_continue_hint();
    }

    fn draw_sky(&self, v: &View, night: f32, sunset: f32) {
        let w = screen_width();
        let h = v.p(Vec2::ZERO).y;
        // a Marseille : grand ciel bleu
        let med = self.sunny();
        let day_top = mix(Color::new(0.35, 0.6, 0.92, 1.0), Color::new(0.12, 0.48, 0.95, 1.0), med);
        let day_bot = mix(Color::new(0.75, 0.88, 0.98, 1.0), Color::new(0.6, 0.85, 1.0, 1.0), med);
        let set_top = Color::new(0.45, 0.35, 0.65, 1.0);
        let set_bot = Color::new(1.0, 0.6, 0.35, 1.0);
        let night_top = Color::new(0.02, 0.03, 0.1, 1.0);
        let night_bot = Color::new(0.1, 0.1, 0.25, 1.0);
        let top = mix(mix(day_top, set_top, sunset), night_top, night);
        let bot = mix(mix(day_bot, set_bot, sunset), night_bot, night);
        const BANDS: usize = 24;
        for i in 0..BANDS {
            let t = i as f32 / BANDS as f32;
            draw_rectangle(0.0, t * h, w, h / BANDS as f32 + 1.0, mix(top, bot, t));
        }
        // Etoiles
        if night > 0.2 {
            for i in 0..90 {
                let x = hash(i, 11) * w;
                let y = hash(i, 12) * h * 0.6;
                let tw = 0.6 + 0.4 * (self.time * 2.0 + i as f32).sin();
                draw_circle(x, y, 1.0 + hash(i, 13) * 1.2, Color::new(1.0, 1.0, 0.95, (night - 0.2) * tw));
            }
        }
        // Soleil / lune
        let sun_y = h * (0.15 + 0.55 * (night + sunset * 0.5).min(1.0));
        if night < 0.9 {
            draw_circle(w * 0.78, sun_y, 38.0, Color::new(1.0, 0.92 - 0.3 * sunset, 0.55 - 0.3 * sunset, 1.0 - night));
        }
        if night > 0.1 {
            let mx = w * 0.2;
            let my = h * 0.18;
            draw_circle(mx, my, 30.0, Color::new(0.97, 0.96, 0.85, night));
            draw_circle(mx + 12.0, my - 6.0, 26.0, mix(top, bot, 0.18));
        }
        // Nuages
        for i in 0..6 {
            let par = (self.pos.x * 6.0 + self.time * 8.0) * 0.5;
            let cx = ((hash(i, 21) * (w + 400.0) - par).rem_euclid(w + 400.0)) - 200.0;
            let cy = h * (0.1 + 0.25 * hash(i, 22));
            let col = mix(Color::new(1.0, 1.0, 1.0, 0.85), Color::new(0.4, 0.4, 0.55, 0.5), night);
            let col = mix(col, Color::new(1.0, 0.75, 0.6, 0.85), sunset * (1.0 - night));
            let col = with_alpha(col, col.a * (1.0 - 0.8 * med)); // le mistral a chasse les nuages
            for k in 0..4 {
                let r = 22.0 + hash(i * 5 + k, 23) * 22.0;
                draw_circle(cx + k as f32 * 30.0, cy + (k as f32 - 1.5).abs() * 6.0, r, col);
            }
        }
    }

    fn draw_background(&self, v: &View, night: f32) {
        let gs = v.p(Vec2::ZERO).y;
        let fog = Color::new(0.75, 0.8, 0.88, 1.0);
        let fogk = |c: Color, k: f32| mix(darker(c, 1.0 - 0.6 * night), mix(fog, Color::new(0.1, 0.12, 0.25, 1.0), night), k);

        // Rive droite au loin (et la fleche Saint-Michel)
        let bank_sy = gs - 1.6 * v.s - v.cam_y * v.s * 0.5;
        let far = v.layer(0.08, 0.28, bank_sy);
        let (l, r) = far.x_range();
        let bw = 5.0;
        let i0 = (l / bw).floor() as i32 - 1;
        let i1 = (r / bw).ceil() as i32 + 1;
        let med = self.sunny();
        for i in i0..=i1 {
            let x0 = i as f32 * bw;
            let fz = zone_at(x0 / 0.08);
            if fz == Zone::Sahara {
                // le Sahara : des dunes a perte de vue
                let dh = 3.0 + hash(i, 53) * 5.0;
                half_ellipse(&far, x0 + bw * 0.5, 0.0, bw * 1.6, dh, fogk(SAND, 0.35));
                half_ellipse(&far, x0 + bw * 1.1, 0.0, bw * 1.1, dh * 0.6, fogk(darker(SAND, 0.88), 0.35));
                continue;
            }
            if fz.city() == City::Algerie {
                // Alger la Blanche, en gradins sur ses collines
                let hill = 5.0 + hash(i, 51) * 6.0;
                half_ellipse(&far, x0 + bw * 0.5, 0.0, bw * 1.5, hill, fogk(Color::new(0.5, 0.6, 0.42, 1.0), 0.5));
                for k in 0..5 {
                    let hx = x0 + hash(i * 5 + k, 54) * bw;
                    let hy = hash(i * 5 + k, 55) * hill * 0.6;
                    let hw = 0.8 + hash(i * 5 + k, 56) * 0.9;
                    rect(&far, hx, hy, hx + hw, hy + 0.9 + hash(i * 5 + k, 57), fogk(Color::new(0.97, 0.97, 0.95, 1.0), 0.4));
                }
                continue;
            }
            if fz.city() == City::Forcalquier {
                // la montagne de Lure, des collines et des cypres
                let lure = 11.0 + hash(i, 51) * 6.0;
                tri(&far, vec2(x0 - bw * 2.5, 0.0), vec2(x0 + bw * 3.5, 0.0), vec2(x0 + bw * 0.5, lure), fogk(Color::new(0.5, 0.58, 0.75, 1.0), 0.6));
                let hill = 2.5 + hash(i, 52) * 3.0;
                half_ellipse(&far, x0 + bw * 0.5, 0.0, bw * 1.4, hill, fogk(Color::new(0.55, 0.62, 0.38, 1.0), 0.45));
                if hash(i, 59) < 0.6 {
                    // un champ de lavande a flanc de colline
                    for k in 0..4 {
                        let yy = 0.2 + k as f32 * hill * 0.14;
                        rect(&far, x0 + bw * 0.1 + k as f32 * 0.4, yy, x0 + bw * 1.1 - k as f32 * 0.3, yy + hill * 0.08, fogk(LAVENDER, 0.35));
                    }
                }
                if hash(i, 58) < 0.5 {
                    let cx = x0 + bw * 0.3;
                    tri(&far, vec2(cx - 0.35, hill * 0.6), vec2(cx + 0.35, hill * 0.6), vec2(cx, hill * 0.6 + 3.2), fogk(Color::new(0.2, 0.35, 0.22, 1.0), 0.4));
                } else {
                    let hh = 1.0 + hash(i, 1);
                    rect(&far, x0 + 1.0, hill * 0.7, x0 + 2.6, hill * 0.7 + hh, fogk(Color::new(0.9, 0.82, 0.66, 1.0), 0.45));
                    tri(&far, vec2(x0 + 0.8, hill * 0.7 + hh), vec2(x0 + 2.8, hill * 0.7 + hh), vec2(x0 + 1.8, hill * 0.7 + hh + 0.6), fogk(TILE, 0.45));
                }
                continue;
            }
            if fz.city() == City::Marseille {
                // Marseille : collines de calcaire, maisons aux toits de tuiles
                let hill = 5.0 + hash(i, 51) * 9.0;
                tri(&far, vec2(x0 - bw * 1.2, 0.0), vec2(x0 + bw * 2.2, 0.0), vec2(x0 + bw * 0.5, hill), fogk(LIMESTONE, 0.5));
                tri(&far, vec2(x0 + bw * 0.5, hill), vec2(x0 + bw * 2.2, 0.0), vec2(x0 + bw * 1.1, 0.0), fogk(darker(LIMESTONE, 0.85), 0.5));
                if hash(i, 52) < 0.6 {
                    tri(&far, vec2(x0 - bw * 0.2, 0.0), vec2(x0 + bw * 1.4, 0.0), vec2(x0 + bw * 0.6, hill * 0.45), fogk(Color::new(0.45, 0.55, 0.35, 1.0), 0.5));
                }
                let hh = 1.2 + hash(i, 1) * 2.2;
                rect(&far, x0, 0.0, x0 + bw * 0.6, hh, fogk(Color::new(0.93, 0.8, 0.6, 1.0), 0.45));
                tri(&far, vec2(x0 - 0.2, hh), vec2(x0 + bw * 0.6 + 0.2, hh), vec2(x0 + bw * 0.3, hh + 0.7), fogk(TILE, 0.45));
                continue;
            }
            let hh = 3.0 + hash(i, 1) * 5.0;
            rect(&far, x0, 0.0, x0 + bw * 0.95, hh, fogk(STONE_SHADE, 0.55));
            tri(&far, vec2(x0, hh), vec2(x0 + bw * 0.95, hh), vec2(x0 + bw * 0.5, hh + 1.5), fogk(SLATE, 0.5));
            if i.rem_euclid(9) == 0 {
                // fleche gothique
                let cx = x0 + bw * 0.5;
                rect(&far, cx - 1.2, 0.0, cx + 1.2, 14.0, fogk(STONE_SHADE, 0.45));
                tri(&far, vec2(cx - 1.2, 14.0), vec2(cx + 1.2, 14.0), vec2(cx, 26.0), fogk(STONE_DARK, 0.45));
            }
        }
        // La Garonne
        let water_top = bank_sy;
        // La Garonne... ou la Mediterranee
        let wc = mix(fogk(self.band_col(), 0.2), Color::new(0.08, 0.08, 0.12, 1.0), night * 0.5);
        draw_rectangle(0.0, water_top, screen_width(), gs - water_top, wc);
        for k in 0..12 {
            let y = water_top + (k as f32 + 0.5) / 12.0 * (gs - water_top);
            let off = (self.pos.x * 0.5 * v.s * (0.3 + k as f32 * 0.05) + k as f32 * 70.0).rem_euclid(140.0);
            let mut x = -off;
            while x < screen_width() {
                draw_line(x, y, x + 40.0, y, 2.0, mix(darker(wc, 0.85), Color::new(1.0, 1.0, 1.0, 1.0), med * 0.18 * (1.0 - night)));
                x += 140.0;
            }
        }

        // Plan intermediaire : facades en pierre blonde et monuments
        let f = 0.35;
        let mid = v.layer(f, 0.55, gs - 0.3 * v.s);
        let (l, r) = mid.x_range();

        // Monuments du quartier
        let zl = ((l - 40.0) / f / ZONE_LEN).floor() as i32;
        let zr = ((r + 40.0) / f / ZONE_LEN).floor() as i32;

        let bw = 8.0;
        let i0 = (l / bw).floor() as i32 - 1;
        let i1 = (r / bw).ceil() as i32 + 1;
        for i in i0..=i1 {
            let x0 = i as f32 * bw;
            let xc = x0 + bw * 0.5;
            let wx = xc / f;
            let zone = zone_at(wx);
            if zone.river() {
                continue;
            }
            let zi = zone_index(wx);
            let lx = (zi as f32 * ZONE_LEN + ZONE_LEN * 0.5) * f;
            let lh = zone.landmark_half();
            if lh > 0.0 && (xc - lx).abs() < lh + bw * 0.5 {
                continue;
            }
            if zone.city() == City::Bordeaux {
                draw_facade(&mid, i, x0, bw, zone, night);
            } else {
                draw_facade_south(&mid, i, x0, bw, zone, night);
            }
        }
        for zi in zl..=zr {
            let zone = zone_of(zi);
            let lx = (zi as f32 * ZONE_LEN + ZONE_LEN * 0.5) * f;
            draw_landmark(&mid, zone, lx, night, self.time);
        }

        // Voile de lointain
        draw_rectangle(0.0, 0.0, screen_width(), gs, Color::new(0.8, 0.85, 0.92, 0.12 * (1.0 - night)));

        // Premier plan du decor : parapet des quais et reverberes
        let (l, r) = v.x_range();
        let mut x = (l / 1.0).floor();
        while x < r + 1.0 {
            let z = zone_at(x);
            if (z.river() && z != Zone::Sahara && z.city() != City::Forcalquier) || z == Zone::Bourse {
                rect(v, x, 0.0, x + 1.02, 0.75, darker(STONE_SHADE, 1.0 - 0.5 * night));
                rect(v, x, 0.75, x + 1.02, 0.85, darker(STONE, 1.0 - 0.5 * night));
                if z == Zone::PontDePierre {
                    // balustres
                    for k in 0..3 {
                        let bx = x + 0.17 + k as f32 * 0.33;
                        ellipse(v, vec2(bx, 0.4), 0.07, 0.28, 0.0, darker(STONE, 0.9 - 0.4 * night));
                    }
                }
            }
            x += 1.0;
        }
        let lamp_gap = 14.0;
        let mut i = (l / lamp_gap).floor() as i32;
        while (i as f32) * lamp_gap < r + 2.0 {
            draw_lamp(v, i as f32 * lamp_gap + 5.0, night);
            i += 1;
        }
    }

    fn draw_ground(&self, v: &View, night: f32) {
        let w = screen_width();
        let h = screen_height();
        let dk = 1.0 - 0.55 * night;
        let gs = v.p(Vec2::ZERO).y;
        let (l, r) = v.x_range();

        // Sous-sol par defaut : dalles de pierre
        draw_rectangle(0.0, gs, w, h - gs, darker(STONE_SHADE, dk));
        let mut x = l.floor();
        while x < r + 1.0 {
            let z = zone_at(x);
            match z {
                Zone::PontDePierre => {
                    // tablier en brique, piles et arches au-dessus de la Garonne
                    rect(v, x, -3.0, x + 1.02, 0.0, darker(BRICK, dk));
                }
                _ => {
                    rect(v, x, -3.0, x + 1.02, -0.35, darker(if (x as i32) % 2 == 0 { STONE } else { STONE_SHADE }, dk));
                    line(v, vec2(x, -0.35), vec2(x, -3.0), 0.02, darker(STONE_DARK, dk));
                }
            }
            x += 1.0;
        }
        // Arches du Pont de pierre
        let arch = 6.0;
        let mut i = (l / arch).floor() as i32;
        while (i as f32) * arch < r + arch {
            let ax = i as f32 * arch + arch * 0.5;
            if zone_at(ax) == Zone::PontDePierre {
                let wcol = darker(GARONNE, dk * 0.9);
                rect(v, ax - 2.2, -3.0, ax + 2.2, -1.5, wcol);
                half_ellipse(v, ax, -1.5, 2.2, 0.85, wcol);
                // medaillon sur la pile
                circ(v, vec2(ax + 3.0, -1.0), 0.22, darker(STONE, dk));
                circ(v, vec2(ax + 3.0, -1.0), 0.15, darker(BRICK, dk));
                rect(v, ax - 3.0, -0.42, ax + 3.0, -0.3, darker(STONE, dk));
            }
            i += 1;
        }
        // Bande du haut : voie de tram engazonnee (ou Miroir d'eau, ou tablier)
        let mut x = l.floor();
        while x < r + 1.0 {
            let z = zone_at(x);
            match z {
                Zone::Bourse => {
                    let shine = 0.5 + 0.5 * ((x * 0.7 + self.time).sin());
                    let c = mix(Color::new(0.45, 0.55, 0.65, 1.0), Color::new(0.75, 0.82, 0.9, 1.0), shine * 0.4);
                    rect(v, x, -0.35, x + 1.02, 0.0, darker(c, dk));
                }
                Zone::PontDePierre => {
                    rect(v, x, -0.3, x + 1.02, 0.0, darker(Color::new(0.4, 0.4, 0.42, 1.0), dk));
                }
                _ if z.city() != City::Bordeaux => {
                    // au soleil : pas de gazon, des paves, de l'herbe seche ou du sable
                    let (a, b) = match (z, z.city()) {
                        (Zone::Sahara, _) => (SAND, Color::new(0.9, 0.74, 0.46, 1.0)),
                        (_, City::Forcalquier) => (Color::new(0.8, 0.76, 0.48, 1.0), Color::new(0.74, 0.72, 0.44, 1.0)),
                        (_, City::Algerie) => (Color::new(0.9, 0.87, 0.8, 1.0), Color::new(0.82, 0.8, 0.74, 1.0)),
                        _ => (Color::new(0.83, 0.72, 0.58, 1.0), Color::new(0.78, 0.66, 0.52, 1.0)),
                    };
                    rect(v, x, -0.35, x + 1.02, 0.0, darker(if (x as i32).rem_euclid(2) == 0 { a } else { b }, dk));
                }
                _ => {
                    rect(v, x, -0.35, x + 1.02, 0.0, darker(Color::new(0.35, 0.6, 0.25, 1.0), dk));
                    for k in 0..4 {
                        let gx = x + 0.12 + k as f32 * 0.25;
                        line(v, vec2(gx, -0.02), vec2(gx + 0.05, 0.08), 0.03, darker(Color::new(0.3, 0.55, 0.2, 1.0), dk));
                    }
                }
            }
            x += 1.0;
        }
        // Rails (seulement la ou roule un tram : Bordeaux et Alger)
        let rail = darker(Color::new(0.65, 0.66, 0.7, 1.0), dk);
        let mut x = l.floor();
        while x < r + 1.0 {
            if matches!(zone_at(x).city(), City::Bordeaux | City::Algerie) {
                rect(v, x, -0.13, x + 1.02, -0.08, rail);
                rect(v, x, -0.31, x + 1.02, -0.26, rail);
            }
            x += 1.0;
        }

        // Brume du Miroir d'eau
        let cycle = (self.time / 9.0).fract();
        if cycle < 0.35 {
            let a = (cycle / 0.35 * PI).sin() * 0.45;
            let mut x = l.floor();
            while x < r + 1.0 {
                if zone_at(x) == Zone::Bourse {
                    for k in 0..2 {
                        let yy = 0.3 + hash(x as i32 * 2 + k, 31) * 1.2 + (self.time + x).sin() * 0.1;
                        ellipse(v, vec2(x + k as f32 * 0.5, yy * 0.7), 0.8, 0.35, 0.0, Color::new(1.0, 1.0, 1.0, a * 0.3));
                    }
                }
                x += 1.0;
            }
        }
    }

    fn draw_decals(&self, v: &View) {
        if !self.gore {
            return;
        }
        for d in &self.decals {
            ellipse(v, vec2(d.x, 0.0), d.w, d.w * 0.25, 0.0, blood(d.shade * 0.5));
        }
    }

    fn draw_weather(&self, v: &View) {
        // Pluie rue Sainte-Catherine
        let (l, r) = v.x_range();
        let zl = zone_at(l) == Zone::SainteCatherine;
        let zr = zone_at(r) == Zone::SainteCatherine;
        if !zl && !zr {
            return;
        }
        let w = screen_width();
        let h = screen_height();
        let split = if zl && zr {
            None
        } else {
            let bound = (zone_index(r) as f32) * ZONE_LEN;
            Some(v.p(vec2(bound, 0.0)).x)
        };
        for i in 0..160 {
            let x = (hash(i, 41) * (w + 100.0) - self.time * 120.0).rem_euclid(w + 100.0) - 50.0;
            let y = (hash(i, 42) * h + self.time * 900.0 * (0.8 + 0.4 * hash(i, 43))).rem_euclid(h);
            if let Some(sx) = split
                && ((zl && x > sx) || (zr && x < sx)) {
                    continue;
                }
            draw_line(x, y, x - 4.0, y + 16.0, 1.5, Color::new(0.75, 0.82, 0.95, 0.5));
        }
    }

    fn draw_particles(&self, v: &View, behind: bool) {
        for p in &self.particles {
            let is_behind = matches!(p.kind, PK::Dust | PK::Water);
            if is_behind != behind {
                continue;
            }
            let a = (p.life / p.max_life).min(1.0);
            match p.kind {
                PK::Blood => circ(v, p.pos, p.size, p.col),
                PK::Water => circ(v, p.pos, p.size, p.col),
                PK::Chunk | PK::Debris => {
                    let c = with_alpha(p.col, (a * 3.0).min(1.0));
                    ellipse(v, p.pos, p.size, p.size * 0.7, p.ang, c);
                }
                PK::Feather => {
                    let c = with_alpha(p.col, (a * 2.0).min(1.0));
                    ellipse(v, p.pos, p.size, p.size * 0.3, p.ang, c);
                    let d = Vec2::from_angle(p.ang) * p.size;
                    line(v, p.pos - d, p.pos + d, 0.012, with_alpha(darker(p.col, 0.6), c.a));
                }
                PK::Fur => {
                    ellipse(v, p.pos, p.size, p.size * 0.35, p.ang, with_alpha(p.col, (a * 2.0).min(1.0)));
                }
                PK::Star => {
                    let c = with_alpha(p.col, a);
                    let d1 = Vec2::from_angle(p.ang) * p.size;
                    let d2 = Vec2::from_angle(p.ang + PI / 2.0) * p.size;
                    line(v, p.pos - d1, p.pos + d1, 0.03, c);
                    line(v, p.pos - d2, p.pos + d2, 0.03, c);
                }
                PK::Dust => circ(v, p.pos, p.size, with_alpha(p.col, p.col.a * a)),
                PK::Eye => {
                    circ(v, p.pos, p.size, WHITE);
                    let d = Vec2::from_angle(p.ang) * p.size * 0.35;
                    circ(v, p.pos + d, p.size * 0.6, Color::new(0.45, 0.75, 0.2, 1.0));
                    circ(v, p.pos + d * 1.3, p.size * 0.3, INK);
                }
            }
        }
    }

    fn draw_pops(&self, v: &View) {
        for p in &self.pops {
            let sp = v.p(p.pos + vec2(0.0, p.t * 0.8));
            let a = (1.0 - p.t).clamp(0.0, 1.0);
            let pulse = 1.0 + 0.25 * (1.0 - (p.t * 8.0).min(1.0));
            let size = (v.s * 0.42 * pulse) as u16;
            comic_text(&p.txt, sp.x, sp.y, size.max(14), with_alpha(p.col, a), Color::new(0.25, 0.02, 0.02, a));
        }
    }

    fn draw_splats(&self) {
        for s in &self.splats {
            let a = (s.life / s.max_life * 1.6).min(0.85);
            let c = Color::new(0.5, 0.0, 0.02, a);
            draw_circle(s.pos.x, s.pos.y, s.r, c);
            for (off, r) in s.sats {
                draw_circle(s.pos.x + off.x, s.pos.y + off.y, r, c);
            }
            let dw = s.r * 0.22;
            draw_rectangle(s.pos.x - dw / 2.0, s.pos.y, dw, s.r * 0.6 + s.drip, c);
            draw_circle(s.pos.x, s.pos.y + s.r * 0.6 + s.drip, dw * 0.8, c);
        }
    }

    // -----------------------------------------------------------------------
    // Acaboulette, le gros matou roux
    // -----------------------------------------------------------------------

    fn draw_cat(&self, v: &View) {
        if self.tank > 0.0 && self.phase == Phase::Running {
            // dans le char : seule la tete depasse de la tourelle
            let blink = self.tank < 2.0 && (self.time * 14.0).sin() < 0.0;
            draw_tank(v, self.pos.x - TANK_SEAT, self.run_phase, self.tank_recoil, Some(blink));
            return;
        }
        if self.invuln > 0.0 && self.phase == Phase::Running && (self.time * 14.0).sin() > 0.3 {
            return;
        }
        if self.death == Some(Death::Mine) && self.gore {
            return; // il n'en reste que des morceaux
        }
        let r = self.body_r();
        let ph = self.run_phase;
        let airborne = !self.on_ground && self.phase == Phase::Running;
        let dead = self.phase == Phase::Over;
        let title = self.phase == Phase::Title;

        let (sx, sy) = match self.death {
            Some(Death::Tram) => (1.6, 0.2),
            Some(Death::KO) | Some(Death::Mine) => (1.15, 0.6),
            None if self.slamming => (0.88, 1.15),
            None => (1.0 + 0.18 * self.squash, 1.0 - 0.22 * self.squash),
        };
        let bob = if self.on_ground && !dead && !title { (ph * 2.0).sin().abs() * 0.06 } else { 0.0 };
        let feet = self.pos;
        let c = feet + vec2(0.0, (0.2 + r * 0.85) * sy + bob);
        let gliding = self.gliding && airborne;
        let lean = if self.slamming {
            -0.25
        } else if gliding {
            (self.vy * 0.08).clamp(-0.25, 0.5) // le nez suit la trajectoire
        } else if airborne {
            (self.vy * 0.02).clamp(-0.2, 0.2)
        } else {
            0.0
        };
        let rot = Vec2::from_angle(lean);
        let at = |lx: f32, ly: f32| c + rot.rotate(vec2(lx * sx, ly * sy));

        // Aura du mode gave
        if self.gave > 0.0 {
            let pulse = 0.5 + 0.5 * (self.time * 12.0).sin();
            let a = if self.gave < 1.5 { 0.25 * (self.time * 10.0).sin().abs() } else { 0.3 };
            ellipse(v, c, r * 2.0 + pulse * 0.15, r * 1.6 + pulse * 0.1, 0.0, Color::new(1.0, 0.8, 0.2, a));
            for k in 0..4 {
                let y = c.y - r * 0.5 + k as f32 * r * 0.35;
                let x = c.x - r * 1.6 - (self.time * 20.0 + k as f32).rem_euclid(1.0) * 1.2;
                line(v, vec2(x, y), vec2(x - 0.8, y), 0.05, Color::new(1.0, 0.85, 0.3, 0.6));
            }
        }

        // Kebab turbo : flammes et grandes trainees
        if self.kebab > 0.0 {
            let blink = self.kebab < 1.5 && (self.time * 12.0).sin() < 0.0;
            if !blink {
                for k in 0..6 {
                    let y = c.y - r * 0.7 + k as f32 * r * 0.3;
                    let len = 1.2 + (self.time * 30.0 + k as f32 * 1.7).sin().abs() * 1.2;
                    let x = c.x - r * 1.3;
                    let col = if k % 2 == 0 { Color::new(1.0, 0.35, 0.05, 0.7) } else { Color::new(1.0, 0.8, 0.1, 0.7) };
                    tri(v, vec2(x, y + 0.12), vec2(x, y - 0.12), vec2(x - len, y), col);
                }
                let pulse = 0.5 + 0.5 * (self.time * 20.0).sin();
                ellipse(v, c, r * 1.9 + pulse * 0.1, r * 1.5, 0.0, Color::new(1.0, 0.3, 0.1, 0.22));
            }
        }

        // Ombre
        if !dead {
            let ground = if self.on_tram { TRAM_TOP } else if self.pos.y > 0.0 && self.on_ground { self.pos.y } else { 0.0 };
            let k = (1.0 - (self.pos.y - ground) / 6.0).clamp(0.3, 1.0);
            ellipse(v, vec2(feet.x, ground + 0.01), r * 1.3 * k, 0.08, 0.0, Color::new(0.0, 0.0, 0.0, 0.25 * k));
        }

        // Queue
        let tail_base = at(-r * 1.1, r * 0.1);
        let mut prev = tail_base;
        let wave = if dead { 0.0 } else { (self.time * 5.0).sin() * 0.35 };
        for k in 1..=6 {
            let t = k as f32 / 6.0;
            let ang = PI * 0.6 + t * 0.9 + wave * t + if self.slamming { -0.8 } else { 0.0 };
            let ang = if dead { PI * 0.98 } else { ang };
            let p = prev + Vec2::from_angle(ang) * 0.17;
            let col = if k % 2 == 0 { GINGER_DARK } else { GINGER };
            seg(v, prev, p, 0.15 - t * 0.04, col);
            prev = p;
        }

        // Deltaplane (triple saut) : l'aile, derriere le chat
        let glide_bar = at(r * 1.15, r * 0.15); // la barre de controle
        let glide_mast = at(r * 0.15, r * 2.02);
        if gliding {
            let open = (self.glide_age / 0.18).min(1.0);
            // l'aile clignote quand elle va se replier
            let a = if self.glide_t > GLIDE_MAX - 0.9 && (self.time * 16.0).sin() < 0.0 { 0.45 } else { 1.0 };
            let nose = at(r * 2.9, r * 1.9);
            let keel = at(-r * 2.5, r * 2.15);
            let tip_far = keel.lerp(at(-r * 1.5, r * 3.5), open);
            let tip_near = keel.lerp(at(-r * 2.3, r * 1.3), open);
            tri(v, nose, keel, tip_far, Color::new(0.75, 0.1, 0.15, a));
            tri(v, nose, keel, tip_near, Color::new(0.95, 0.2, 0.2, a));
            tri(v, nose, keel.lerp(tip_near, 0.45), tip_near, Color::new(1.0, 0.85, 0.2, a));
            let tube = Color::new(0.85, 0.85, 0.9, a);
            line(v, nose, keel, 0.06, tube);
            line(v, nose, tip_far, 0.05, tube);
            line(v, nose, tip_near, 0.05, tube);
            // trapeze et sangle du harnais
            line(v, glide_mast, glide_bar, 0.05, tube);
            line(v, glide_mast, at(-r * 0.3, r * 0.9), 0.04, Color::new(0.15, 0.15, 0.2, a));
        }

        // Pattes du fond
        let leg = |front: bool, far: bool| -> (Vec2, Vec2) {
            let hx = if front { r * 0.7 } else { -r * 0.75 };
            let hip = at(hx + if far { -0.08 } else { 0.08 }, -r * 0.45);
            let mut foot;
            if dead {
                foot = hip + vec2(if front { 0.25 } else { -0.25 }, -0.12);
            } else if gliding {
                // pattes avant sur la barre, pattes arriere qui trainent
                foot = if front { glide_bar } else { hip + rot.rotate(vec2(-0.45, -0.08)) };
            } else if airborne || self.slamming {
                let ext = if self.slamming { 0.1 } else { 0.4 };
                foot = hip + vec2(if front { ext } else { -ext }, -0.22);
            } else if title {
                foot = vec2(hip.x, feet.y);
            } else {
                let p = ph + if front { 0.0 } else { PI } + if far { 0.9 } else { 0.0 };
                foot = vec2(hip.x + p.sin() * 0.28, feet.y + p.cos().max(0.0) * 0.16);
                foot.y = foot.y.min(hip.y - 0.05);
            }
            (hip, foot)
        };
        for front in [false, true] {
            let (hip, foot) = leg(front, true);
            seg(v, hip, foot, 0.19, GINGER_DARK);
            ellipse(v, foot, 0.11, 0.07, 0.0, darker(CREAM, 0.85));
        }

        // Corps
        ellipse(v, c, r * 1.25 * sx, r * sy, lean, GINGER);
        ellipse(v, at(-r * 0.1, r * 0.35), r * 1.05, r * 0.5, lean, GINGER_SHADE);
        // Rayures
        for k in 0..5 {
            let lx = -r * 0.85 + k as f32 * r * 0.38;
            let top = at(lx, r * 0.92);
            let low = at(lx - 0.06, r * 0.35);
            seg(v, top, low, 0.07, GINGER_DARK);
        }
        // Ventre (il ballotte)
        let jig = if self.on_ground && !dead { (ph * 2.0).cos() * 0.03 * (1.0 + self.fat * 0.5) } else { 0.0 };
        ellipse(v, at(r * 0.15, -r * 0.4 + jig), r * 0.9, r * 0.52, lean, CREAM);
        if self.gore && self.bloody > 0.05 {
            ellipse(v, at(r * 0.45, -r * 0.5 + jig), r * 0.35, r * 0.2, lean, with_alpha(blood(0.2), self.bloody * 0.8));
            ellipse(v, at(-r * 0.1, -r * 0.6 + jig), r * 0.2, r * 0.12, lean, with_alpha(blood(0.6), self.bloody * 0.7));
        }

        // Pattes de devant
        for front in [false, true] {
            let (hip, foot) = leg(front, false);
            seg(v, hip, foot, 0.2, GINGER);
            ellipse(v, foot, 0.12, 0.075, 0.0, CREAM);
            if self.gore && self.bloody > 0.1 {
                ellipse(v, foot, 0.1, 0.05, 0.0, with_alpha(blood(0.3), self.bloody * 0.9));
            }
        }

        // Tete
        let hr = 0.27 + 0.035 * self.fat;
        let hc = at(r * 1.08, r * 0.5);
        let (hsx, hsy) = match self.death {
            Some(Death::Tram) => (1.4, 0.35),
            _ => (1.0, 1.0),
        };
        let hp = |lx: f32, ly: f32| hc + vec2(lx * hsx, ly * hsy);
        // oreilles
        for (ex, sgn) in [(-0.45, -1.0f32), (0.35, 1.0)] {
            let base_a = hp(hr * ex - 0.12, hr * 0.6);
            let base_b = hp(hr * ex + 0.14, hr * 0.75);
            let tip = hp(hr * ex + 0.02 * sgn, hr * 1.35);
            tri(v, base_a, base_b, tip, GINGER);
            tri(v, base_a + vec2(0.04, 0.03), base_b + vec2(-0.03, 0.0), tip + vec2(0.0, -0.08), NOSE);
        }
        ellipse(v, hc, hr * 1.12 * hsx, hr * 0.95 * hsy, 0.0, GINGER);
        // le "M" sur le front
        for k in 0..3 {
            let lx = -hr * 0.25 + k as f32 * hr * 0.25;
            seg(v, hp(lx, hr * 0.85), hp(lx + 0.02, hr * 0.5), 0.045, GINGER_DARK);
        }
        // joues / museau
        ellipse(v, hp(hr * 0.55, -hr * 0.3), hr * 0.5 * hsx, hr * 0.36 * hsy, 0.0, CREAM);
        tri(v, hp(hr * 0.85, -hr * 0.02), hp(hr * 1.08, -hr * 0.02), hp(hr * 0.98, -hr * 0.18), NOSE);
        if self.gore && self.bloody > 0.05 {
            ellipse(v, hp(hr * 0.7, -hr * 0.5), hr * 0.32, hr * 0.12, 0.0, with_alpha(blood(0.4), self.bloody));
        }
        // yeux
        let eye = hp(hr * 0.35, hr * 0.2);
        if dead {
            let d = 0.08;
            line(v, eye - vec2(d, d), eye + vec2(d, d), 0.035, INK);
            line(v, eye + vec2(-d, d), eye + vec2(d, -d), 0.035, INK);
        } else {
            let wide = airborne || self.mighty();
            ellipse(v, eye, 0.11, if wide { 0.12 } else { 0.1 }, 0.0, WHITE);
            ellipse(v, eye + vec2(0.03, 0.0), 0.075, 0.09, 0.0, Color::new(0.45, 0.75, 0.2, 1.0));
            ellipse(v, eye + vec2(0.04, 0.0), 0.022, 0.07, 0.0, INK);
            circ(v, eye + vec2(0.06, 0.04), 0.018, WHITE);
            if !wide {
                // paupiere satisfaite
                half_ellipse(v, eye.x, eye.y + 0.01, 0.125, 0.12, GINGER);
                line(v, eye + vec2(-0.12, 0.01), eye + vec2(0.12, 0.01), 0.025, GINGER_DARK);
            }
        }
        // bouche & moustaches
        let m = hp(hr * 0.95, -hr * 0.28);
        if self.kebab > 0.0 && !dead {
            ellipse(v, m + vec2(-0.03, -0.04), 0.07, 0.06, 0.0, Color::new(0.4, 0.05, 0.08, 1.0));
            let wag = (self.time * 25.0).sin() * 0.03;
            seg(v, m + vec2(-0.02, -0.06), m + vec2(-0.12 + wag, -0.2), 0.09, NOSE);
        } else if self.slamming || self.gave > 0.0 {
            ellipse(v, m + vec2(-0.03, -0.04), 0.07, 0.06, 0.0, Color::new(0.4, 0.05, 0.08, 1.0));
        } else {
            line(v, hp(hr * 0.98, -hr * 0.18), m, 0.02, INK);
            line(v, m, m + vec2(-0.08, 0.02), 0.02, INK);
        }
        for k in 0..3 {
            let o = hp(hr * 0.75, -hr * (0.22 + k as f32 * 0.08));
            let tipv = o + vec2(0.42, 0.08 - k as f32 * 0.09);
            line(v, o, tipv, 0.012, Color::new(1.0, 1.0, 1.0, 0.9));
        }

        // Etoiles du KO
        if matches!(self.death, Some(Death::KO) | Some(Death::Mine)) {
            for k in 0..4 {
                let a = self.time * 4.0 + k as f32 * TAU / 4.0;
                let p = hc + vec2(a.cos() * 0.45, 0.45 + a.sin() * 0.12);
                let sp = v.p(p);
                draw_poly(sp.x, sp.y, 5, v.l(0.07), a.to_degrees(), YELLOW);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Interface
    // -----------------------------------------------------------------------

    fn draw_hud(&self) {
        let w = screen_width();
        let h = screen_height();
        let panel = Color::new(0.2, 0.03, 0.07, 0.72);

        if self.phase == Phase::Title {
            self.draw_title();
            return;
        }

        round_rect(12.0, 12.0, 290.0, 116.0, 10.0, panel);
        draw_text(format!("Score    : {}", self.total()), 24.0, 44.0, 32.0, WHITE);
        draw_text(format!("Distance : {} m", self.pos.x.max(0.0) as u32), 24.0, 74.0, 24.0, LIGHTGRAY);
        draw_text(format!("Record   : {}", self.best), 24.0, 104.0, 24.0, Color::new(1.0, 0.85, 0.2, 1.0));

        // Vies et embonpoint
        round_rect(w - 262.0, 12.0, 250.0, 84.0, 10.0, panel);
        draw_text("Vies", w - 250.0, 42.0, 22.0, WHITE);
        for k in 0..LIVES {
            let cx = w - 180.0 + k as f32 * 34.0;
            let col = if k < self.lives { GINGER } else { Color::new(0.3, 0.3, 0.3, 0.6) };
            draw_circle(cx, 36.0, 11.0, col);
            draw_triangle(vec2(cx - 11.0, 32.0), vec2(cx - 3.0, 27.0), vec2(cx - 9.0, 20.0), col);
            draw_triangle(vec2(cx + 11.0, 32.0), vec2(cx + 3.0, 27.0), vec2(cx + 9.0, 20.0), col);
        }
        draw_text("Gras", w - 250.0, 80.0, 22.0, WHITE);
        let bx = w - 190.0;
        let bw = 168.0;
        round_rect(bx, 62.0, bw, 22.0, 6.0, Color::new(0.3, 0.3, 0.3, 0.5));
        let t = self.fat / MAX_FAT;
        if t > 0.0 {
            let fill = mix(Color::new(1.0, 0.75, 0.35, 1.0), Color::new(0.85, 0.25, 0.05, 1.0), t);
            round_rect(bx, 62.0, (bw * t).max(12.0), 22.0, 6.0, fill);
        }
        if self.fat >= MAX_FAT {
            draw_text("MAX", bx + bw / 2.0 - 16.0, 79.0, 20.0, WHITE);
        }
        // Jauges des bonus : en bas, au milieu (empilees si les deux sont actives)
        if self.phase == Phase::Running {
            let mut slot = 0;
            if self.tank > 0.0 {
                bottom_gauge("CHAR D'ASSAUT", self.tank / TANK_TIME, self.tank < 2.0, Color::new(0.14, 0.2, 0.08, 0.85), Color::new(0.55, 0.8, 0.25, 1.0), slot, self.time);
                slot += 1;
            }
            if self.kebab > 0.0 {
                bottom_gauge("KEBAB TURBO", self.kebab / KEBAB_TIME, self.kebab < 1.5, Color::new(0.55, 0.12, 0.02, 0.85), Color::new(1.0, 0.45, 0.1, 1.0), slot, self.time);
                slot += 1;
            }
            if self.gave > 0.0 {
                bottom_gauge("MODE GAVÉ", self.gave / GAVE_TIME, self.gave < 1.5, Color::new(0.5, 0.35, 0.0, 0.85), Color::new(1.0, 0.85, 0.2, 1.0), slot.min(1), self.time);
            }
        }

        // Combo
        if self.combo >= 2 && self.phase == Phase::Running {
            let s = 30 + (self.combo.min(20) * 2) as u16;
            comic_text(&format!("COMBO x{}", self.combo), 157.0, 165.0, s, Color::new(1.0, 0.85, 0.15, 1.0), Color::new(0.45, 0.02, 0.02, 1.0));
        }

        // Annonce du tram (fleche au bord droit)
        if self.phase == Phase::Running {
            let v = self.view();
            for t in &self.trams {
                let sx = v.p(vec2(t.x, 0.0)).x;
                if t.warned && sx > w && t.x - self.pos.x < 40.0 {
                    let y = v.p(vec2(0.0, 1.5)).y;
                    let blink = (self.time * 10.0).sin() > 0.0;
                    let col = if blink { Color::new(1.0, 0.85, 0.1, 1.0) } else { Color::new(0.9, 0.1, 0.1, 1.0) };
                    draw_triangle(vec2(w - 10.0, y), vec2(w - 50.0, y - 26.0), vec2(w - 50.0, y + 26.0), col);
                    let label = self.tram_name().to_uppercase();
                    let label = label.as_str();
                    let tw = measure_text(label, None, 26, 1.0).width;
                    draw_text(label, w - 60.0 - tw, y + 9.0, 26.0, col);
                }
            }
        }

        // Aide de depart
        if self.phase == Phase::Running && self.time < 9.0 && !self.mighty() {
            let a = (9.0 - self.time).min(1.0);
            let hint = "Espace / clic : sauter (2x, puis DELTAPLANE)   -   Bas / clic droit : PLAT VENTRE";
            let size = 22u16;
            let tw = measure_text(hint, None, size, 1.0).width;
            round_rect(w / 2.0 - tw / 2.0 - 12.0, h - 54.0, tw + 24.0, 36.0, 10.0, Color::new(0.0, 0.0, 0.0, 0.5 * a));
            draw_text(hint, w / 2.0 - tw / 2.0, h - 29.0, size as f32, Color::new(1.0, 1.0, 1.0, a));
        }

        let gore_txt = if self.gore { "Gore : OUI (G)" } else { "Gore : non (G)" };
        draw_text(gore_txt, w - 150.0, h - 14.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.7));

        // Message : un seul a la fois, en fondu (les suivants attendent leur tour)
        if !(self.phase == Phase::Over && self.over_t > 0.8) {
            let base = 200.0;
            if let Some(m) = self.msgs.first() {
                let fade_in = (m.age / 0.3).min(1.0);
                let fade_out = ((MSG_LIFE - m.age) / MSG_FADE).clamp(0.0, 1.0);
                let a = fade_in * fade_out;
                let y = base;
                let tw = measure_text(&m.title, None, 42, 1.0).width.max(measure_text(&m.sub, None, 25, 1.0).width);
                round_rect(w / 2.0 - tw / 2.0 - 18.0, y - 30.0, tw + 36.0, if m.sub.is_empty() { 52.0 } else { 88.0 }, 12.0, Color::new(0.15, 0.02, 0.06, 0.6 * a));
                comic_text(&m.title, w / 2.0, y, 42, Color::new(1.0, 1.0, 1.0, a), Color::new(0.35, 0.03, 0.1, a));
                if !m.sub.is_empty() {
                    comic_text(&m.sub, w / 2.0, y + 40.0, 25, Color::new(1.0, 0.9, 0.6, a), Color::new(0.2, 0.02, 0.05, a));
                }
            }
        }

        if self.phase == Phase::Over && self.over_t > 0.8 {
            self.draw_game_over();
        }
    }

    fn draw_title(&self) {
        let w = screen_width();
        let h = screen_height();
        let bw = 640.0f32.min(w - 32.0);
        let bx = w / 2.0 - bw / 2.0;
        round_rect(bx, 40.0, bw, 360.0, 16.0, Color::new(0.2, 0.03, 0.07, 0.8));
        let wob = (self.time * 3.0).sin() * 3.0;
        comic_text("ACABOULETTE", w / 2.0, 100.0 + wob, 80, GINGER, Color::new(0.3, 0.05, 0.02, 1.0));
        let city = CITIES[start_city()];
        let town = if city == City::Algerie { "Alger" } else { city.name() };
        centered(&format!("Le gros matou roux qui écrabouille {town}"), 158.0, 28, Color::new(1.0, 0.9, 0.7, 1.0));
        let intro = format!("Il a la dalle, et {town} regorge de casse-croûte :");
        let lines = [
            intro.as_str(),
            "pigeons, rats, chiens... il gobe tout ce qui passe !",
            "Plus il mange, plus il grossit... jusqu'à la pause",
            "litière. Et là : MODE GAVÉ, il défonce tout. Tarpin !",
            "Un kebab ? TURBO. Un barrage de CRS ? BONUS.",
            "Gare au mobilier, aux voitures, au tram et aux MINES.",
        ];
        for (i, l) in lines.iter().enumerate() {
            centered(l, 206.0 + i as f32 * 28.0, 22, Color::new(1.0, 1.0, 1.0, 0.9));
        }
        if self.best > 0 {
            centered(&format!("Record : {}", self.best), 380.0, 24, Color::new(1.0, 0.85, 0.2, 1.0));
        }
        // Choix de la ville de depart
        let sel = start_city();
        let hover = city_button_at(mouse_position());
        for (k, (bx, by, bw, bh)) in city_buttons().into_iter().enumerate() {
            let on = k == sel;
            let lit = on || hover == Some(k);
            round_rect(bx - 3.0, by - 3.0, bw + 6.0, bh + 6.0, 13.0, if on { Color::new(1.0, 0.85, 0.2, 1.0) } else { Color::new(0.1, 0.0, 0.03, 0.9) });
            round_rect(bx, by, bw, bh, 10.0, if lit { Color::new(0.5, 0.1, 0.2, 1.0) } else { Color::new(0.28, 0.05, 0.11, 0.95) });
            let size = if bw < 150.0 { 20 } else { 26 };
            comic_text(CITIES[k].name(), bx + bw / 2.0, by + bh / 2.0, size, if lit { WHITE } else { Color::new(1.0, 0.9, 0.75, 1.0) }, Color::new(0.1, 0.0, 0.03, 1.0));
        }
        let a = 0.6 + 0.4 * (self.time * 4.0).sin();
        comic_text("Clique sur ta ville de départ !", w / 2.0, h * 0.9, 34, Color::new(1.0, 1.0, 1.0, a), Color::new(0.3, 0.02, 0.05, a));
        centered("(ou flèches gauche / droite, puis Espace)", h * 0.9 + 34.0, 20, Color::new(1.0, 1.0, 1.0, 0.75));
        let gore_txt = if self.gore { "Gore : OUI (G)" } else { "Gore : non (G)" };
        draw_text(gore_txt, w - 150.0, h - 14.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.7));
    }

    fn draw_game_over(&self) {
        let w = screen_width();
        let h = screen_height();
        let bw = 700.0f32.min(w - 32.0);
        let bh = 300.0;
        let bx = w / 2.0 - bw / 2.0;
        let by = h * 0.38;
        round_rect(bx, by, bw, bh, 16.0, Color::new(0.1, 0.02, 0.04, 0.85));
        let title = match self.death {
            Some(Death::Tram) => "EN CRÊPE !",
            Some(Death::Mine) => "EN MORCEAUX !",
            _ => "TARPIN FINI !",
        };
        comic_text(title, w / 2.0, by + 46.0, 56, Color::new(1.0, 0.3, 0.2, 1.0), Color::new(0.2, 0.0, 0.0, 1.0));
        let y0 = by + 100.0;
        centered(&format!("Score : {}   ({} m)", self.total(), self.pos.x.max(0.0) as u32), y0, 30, WHITE);
        let k = self.kills;
        centered(
            &format!("Pigeons {}  -  Rats {}  -  Chiens {}  -  CRS {}  -  Mouettes {}", k[0], k[1], k[2], k[3], k[4]),
            y0 + 36.0,
            22,
            LIGHTGRAY,
        );
        centered(&format!("Gras max atteint : {} fois", self.gave_count), y0 + 64.0, 22, LIGHTGRAY);
        if self.new_record {
            let a = 0.6 + 0.4 * (self.time * 6.0).sin();
            comic_text("NOUVEAU RECORD ! TARPIN !", w / 2.0, y0 + 108.0, 34, Color::new(1.0, 0.85, 0.15, a), Color::new(0.4, 0.05, 0.0, a));
        } else {
            centered(&format!("Record : {}", self.best), y0 + 108.0, 26, Color::new(1.0, 0.85, 0.2, 1.0));
        }
        if self.over_t > 1.2 {
            centered("Clic / Espace : rejouer  -  V : autre ville  -  Échap : quitter", by + bh - 18.0, 22, Color::new(1.0, 1.0, 1.0, 0.85));
        }
    }
}

// ---------------------------------------------------------------------------
// Decor
// ---------------------------------------------------------------------------

fn window_col(night: f32, lit: bool) -> Color {
    if night > 0.3 && lit {
        Color::new(1.0, 0.85, 0.45, 1.0)
    } else {
        mix(Color::new(0.35, 0.42, 0.52, 1.0), Color::new(0.08, 0.09, 0.15, 1.0), night)
    }
}

/// Immeuble bordelais : pierre blonde, hautes fenetres, balcon en fer forge,
/// toit d'ardoise en mansarde.
fn draw_facade(v: &View, i: i32, x0: f32, bw: f32, zone: Zone, night: f32) {
    let dk = 1.0 - 0.6 * night;
    let floors = 3 + (hash(i, 2) * 2.0) as i32;
    let fh = 3.2;
    let top = floors as f32 * fh;
    let tint = 0.92 + hash(i, 3) * 0.1;
    let stone = darker(STONE, dk * tint);
    rect(v, x0, 0.0, x0 + bw - 0.05, top, stone);
    // corniches
    for f in 1..=floors {
        rect(v, x0 - 0.1, f as f32 * fh - 0.25, x0 + bw + 0.05, f as f32 * fh, darker(STONE_SHADE, dk));
    }
    // mansarde
    let slate = darker(SLATE, dk);
    tri(v, vec2(x0, top), vec2(x0 + 0.8, top + 2.4), vec2(x0 + 0.8, top), slate);
    tri(v, vec2(x0 + bw - 0.05, top), vec2(x0 + bw - 0.85, top + 2.4), vec2(x0 + bw - 0.85, top), slate);
    rect(v, x0 + 0.8, top, x0 + bw - 0.85, top + 2.4, slate);
    // lucarnes
    for k in 0..2 {
        let lx = x0 + 2.0 + k as f32 * 3.0;
        rect(v, lx, top + 0.3, lx + 0.9, top + 1.7, darker(STONE, dk));
        rect(v, lx + 0.15, top + 0.45, lx + 0.75, top + 1.4, window_col(night, hash(i * 7 + k, 4) < 0.4));
    }
    // cheminee
    rect(v, x0 + bw * 0.5, top + 1.6, x0 + bw * 0.5 + 0.6, top + 3.0, darker(STONE_SHADE, dk));
    // fenetres
    for f in 0..floors {
        for k in 0..3 {
            let wx = x0 + 0.9 + k as f32 * 2.3;
            let wy = f as f32 * fh + 0.6;
            if f == 0 {
                // rez-de-chaussee : arcades de boutiques
                let shop = zone == Zone::SainteCatherine || zone == Zone::Chartrons;
                let col = if shop {
                    let colors = [Color::new(0.55, 0.1, 0.18, 1.0), Color::new(0.15, 0.3, 0.45, 1.0), Color::new(0.2, 0.4, 0.25, 1.0)];
                    darker(colors[((hash(i * 3 + k, 5) * 3.0) as usize).min(2)], dk)
                } else {
                    window_col(night, hash(i * 3 + k, 6) < 0.3)
                };
                rect(v, wx - 0.1, 0.0, wx + 1.4, 2.2, col);
                half_ellipse(v, wx + 0.65, 2.2, 0.75, 0.5, col);
                if shop {
                    rect(v, wx, 0.3, wx + 1.3, 1.9, window_col(night, true));
                }
            } else {
                rect(v, wx, wy, wx + 1.2, wy + 2.0, window_col(night, hash(i * 11 + f * 3 + k, 7) < 0.35));
                line(v, vec2(wx + 0.6, wy), vec2(wx + 0.6, wy + 2.0), 0.06, darker(STONE, dk));
                // mascaron au-dessus de la fenetre
                circ(v, vec2(wx + 0.6, wy + 2.3), 0.18, darker(STONE_SHADE, dk));
                if f == 1 {
                    // balcon fer forge
                    rect(v, wx - 0.2, wy - 0.1, wx + 1.4, wy + 0.05, darker(INK, 1.0));
                    for b in 0..6 {
                        let bx = wx - 0.1 + b as f32 * 0.28;
                        line(v, vec2(bx, wy), vec2(bx, wy + 0.7), 0.04, INK);
                    }
                    line(v, vec2(wx - 0.2, wy + 0.7), vec2(wx + 1.4, wy + 0.7), 0.06, INK);
                }
            }
        }
    }
    // Bouteilles de vin dans la vitrine aux Chartrons
    if zone == Zone::Chartrons && hash(i, 8) < 0.5 {
        for b in 0..4 {
            let bx = x0 + 1.0 + b as f32 * 0.3;
            rect(v, bx, 0.3, bx + 0.18, 0.9, darker(Color::new(0.15, 0.25, 0.1, 1.0), dk));
            rect(v, bx + 0.05, 0.9, bx + 0.13, 1.15, darker(Color::new(0.15, 0.25, 0.1, 1.0), dk));
        }
    }
}

fn draw_landmark(v: &View, zone: Zone, x: f32, night: f32, time: f32) {
    let dk = 1.0 - 0.6 * night;
    let stone = darker(STONE, dk);
    let shade = darker(STONE_SHADE, dk);
    let slate = darker(SLATE, dk);
    match zone {
        Zone::Chartrons => {
            // Saint-Louis-des-Chartrons et ses deux fleches
            rect(v, x - 6.0, 0.0, x + 6.0, 10.0, stone);
            tri(v, vec2(x - 6.0, 10.0), vec2(x + 6.0, 10.0), vec2(x, 13.0), shade);
            for sx in [-4.0, 4.0] {
                rect(v, x + sx - 1.4, 0.0, x + sx + 1.4, 16.0, shade);
                rect(v, x + sx - 0.5, 11.0, x + sx + 0.5, 14.0, window_col(night, false));
                tri(v, vec2(x + sx - 1.4, 16.0), vec2(x + sx + 1.4, 16.0), vec2(x + sx, 25.0), slate);
            }
            circ(v, vec2(x, 7.5), 1.3, window_col(night, true));
            half_ellipse(v, x, 0.0, 1.5, 4.0, darker(INK, 1.5));
        }
        Zone::Bourse => {
            // Place de la Bourse : facade classique, pavillon central, Trois Graces
            for k in -4..4 {
                let px = x + k as f32 * 6.0;
                rect(v, px, 0.0, px + 5.95, 9.0, stone);
                rect(v, px - 0.05, 8.6, px + 6.0, 9.2, shade);
                for f in 0..2 {
                    for j in 0..2 {
                        let wx = px + 1.0 + j as f32 * 2.6;
                        let wy = 1.0 + f as f32 * 4.0;
                        rect(v, wx, wy, wx + 1.3, wy + 2.6, window_col(night, hash(k * 4 + f * 2 + j, 9) < 0.5));
                        if f == 0 {
                            half_ellipse(v, wx + 0.65, wy + 2.6, 0.65, 0.5, window_col(night, false));
                        }
                    }
                }
                // toit mansarde
                rect(v, px + 0.3, 9.2, px + 5.65, 11.2, slate);
            }
            // pavillon central et fronton
            rect(v, x - 5.0, 0.0, x + 5.0, 11.0, stone);
            tri(v, vec2(x - 5.4, 11.0), vec2(x + 5.4, 11.0), vec2(x, 13.6), shade);
            tri(v, vec2(x - 4.2, 11.3), vec2(x + 4.2, 11.3), vec2(x, 13.0), stone);
            for k in 0..4 {
                let cx = x - 3.6 + k as f32 * 2.4;
                rect(v, cx, 0.0, cx + 0.5, 10.6, shade);
            }
            half_ellipse(v, x, 14.0, 2.5, 2.5, slate);
            // fontaine des Trois Graces
            rect(v, x - 1.5, 0.0, x + 1.5, 0.8, shade);
            rect(v, x - 0.25, 0.8, x + 0.25, 2.5, shade);
            for k in -1..=1 {
                ellipse(v, vec2(x + k as f32 * 0.4, 3.2), 0.18, 0.7, 0.0, darker(Color::new(0.25, 0.3, 0.28, 1.0), dk));
                circ(v, vec2(x + k as f32 * 0.4, 4.0), 0.16, darker(Color::new(0.25, 0.3, 0.28, 1.0), dk));
            }
            let jet = 0.4 + 0.1 * (time * 6.0).sin();
            ellipse(v, vec2(x, 1.1), 1.4, jet, 0.0, Color::new(0.8, 0.9, 1.0, 0.5));
        }
        Zone::PontDePierre => {}
        Zone::GrosseCloche => {
            // La Grosse Cloche : porte, deux tours rondes, cloche et horloge
            rect(v, x - 3.0, 0.0, x + 3.0, 12.0, stone);
            half_ellipse(v, x, 0.0, 1.8, 4.5, darker(INK, 1.6));
            for sx in [-4.0, 4.0] {
                rect(v, x + sx - 1.8, 0.0, x + sx + 1.8, 13.0, shade);
                for k in 0..3 {
                    rect(v, x + sx - 0.25, 3.0 + k as f32 * 3.0, x + sx + 0.25, 4.2 + k as f32 * 3.0, window_col(night, false));
                }
                tri(v, vec2(x + sx - 2.1, 13.0), vec2(x + sx + 2.1, 13.0), vec2(x + sx, 20.0), slate);
                line(v, vec2(x + sx, 20.0), vec2(x + sx, 21.0), 0.12, darker(GOLD, dk));
            }
            // horloge
            circ(v, vec2(x, 9.0), 1.6, darker(Color::new(0.12, 0.14, 0.3, 1.0), dk));
            circ(v, vec2(x, 9.0), 1.3, darker(Color::new(0.95, 0.92, 0.8, 1.0), dk));
            let a = time * 0.3;
            line(v, vec2(x, 9.0), vec2(x, 9.0) + Vec2::from_angle(PI / 2.0 - a) * 1.0, 0.1, INK);
            line(v, vec2(x, 9.0), vec2(x, 9.0) + Vec2::from_angle(PI / 2.0 - a / 12.0) * 0.7, 0.14, INK);
            // beffroi et la cloche
            rect(v, x - 2.0, 12.0, x + 2.0, 15.0, shade);
            rect(v, x - 1.4, 12.2, x + 1.4, 14.6, darker(INK, 1.4));
            let swing = (time * 2.0).sin() * 0.15;
            ellipse(v, vec2(x + swing, 13.3), 0.8, 0.9, swing, darker(Color::new(0.6, 0.45, 0.2, 1.0), dk));
            tri(v, vec2(x - 2.4, 15.0), vec2(x + 2.4, 15.0), vec2(x, 17.0), slate);
        }
        Zone::SainteCatherine => {
            // Le Grand-Theatre, au bout de la rue Sainte-Catherine
            rect(v, x - 12.0, 0.0, x + 12.0, 10.0, stone);
            rect(v, x - 12.2, 9.5, x + 12.2, 10.6, shade);
            for k in 0..12 {
                let cx = x - 11.0 + k as f32 * 2.0;
                rect(v, cx, 1.0, cx + 0.7, 9.4, shade);
                rect(v, cx - 0.15, 9.0, cx + 0.85, 9.5, stone);
                // statue
                ellipse(v, vec2(cx + 0.35, 11.4), 0.25, 0.7, 0.0, stone);
                circ(v, vec2(cx + 0.35, 12.3), 0.22, stone);
            }
            rect(v, x - 12.0, 0.0, x + 12.0, 1.0, shade);
        }
        Zone::CiteDuVin => {
            // La Cité du Vin : la carafe doree qui tourbillonne
            let gold = darker(Color::new(0.85, 0.7, 0.4, 1.0), dk);
            let gold2 = darker(Color::new(0.7, 0.55, 0.3, 1.0), dk);
            for k in 0..14 {
                let t = k as f32 / 13.0;
                let y = 1.0 + t * 13.0;
                let wdt = 6.5 * (1.0 - t * 0.75) * (1.0 + 0.15 * (t * 5.0).sin());
                let ox = (t * 4.0).sin() * 1.2;
                ellipse(v, vec2(x + ox, y), wdt, 1.4, (t * 3.0).sin() * 0.3, if k % 2 == 0 { gold } else { gold2 });
            }
            rect(v, x - 0.3, 14.0, x + 0.3, 17.0, gold2);
            for k in 0..6 {
                let y = 2.0 + k as f32 * 1.8;
                line(v, vec2(x - 5.0 + k as f32 * 0.6, y), vec2(x + 4.0 - k as f32 * 0.4, y + 0.6), 0.08, window_col(night, k % 2 == 0));
            }
        }
        Zone::Chaban => {
            // Pont Chaban-Delmas : quatre pylones, le tablier leve
            let steel = darker(Color::new(0.72, 0.75, 0.78, 1.0), dk);
            let lift = 6.0 + 2.0 * (time * 0.2).sin();
            for sx in [-14.0, 14.0] {
                rect(v, x + sx - 1.0, 0.0, x + sx + 1.0, 18.0, steel);
                rect(v, x + sx - 0.4, 2.0, x + sx + 0.4, 17.0, darker(steel, 0.8));
                tri(v, vec2(x + sx - 1.0, 18.0), vec2(x + sx + 1.0, 18.0), vec2(x + sx, 19.5), steel);
                // tablier fixe vers la rive
                rect(v, x + sx * 2.0 - sx.signum() * 0.0, 1.5, x + sx, 2.1, steel);
            }
            rect(v, x - 13.0, lift, x + 13.0, lift + 0.9, steel);
            for k in 0..9 {
                let cx = x - 12.0 + k as f32 * 3.0;
                line(v, vec2(cx, lift), vec2(cx + 1.5, lift + 0.9), 0.08, darker(steel, 0.7));
            }
            for sx in [-14.0, 14.0] {
                line(v, vec2(x + sx, 18.0), vec2(x + sx * 0.6, lift + 0.9), 0.06, darker(steel, 0.6));
            }
            if night > 0.3 {
                circ(v, vec2(x - 14.0, 19.0), 0.3, Color::new(1.0, 0.2, 0.2, night));
                circ(v, vec2(x + 14.0, 19.0), 0.3, Color::new(1.0, 0.2, 0.2, night));
            }
        }
        Zone::VieuxPort => {
            // Le Vieux-Port : fort Saint-Jean, voiliers et barques de pecheurs
            let pink = darker(Color::new(0.9, 0.75, 0.62, 1.0), dk);
            rect(v, x - 34.0, 0.0, x - 22.0, 6.0, pink);
            rect(v, x - 30.0, 6.0, x - 25.0, 13.0, darker(pink, 0.92));
            for k in 0..5 {
                rect(v, x - 30.0 + k as f32 * 1.1, 13.0, x - 29.4 + k as f32 * 1.1, 13.8, darker(pink, 0.92));
            }
            rect(v, x - 28.0, 9.0, x - 27.0, 10.6, window_col(night, false));
            for k in -6..=7 {
                let bx = x + k as f32 * 6.5 + hash(k, 61) * 2.0;
                if bx < x - 21.0 {
                    continue;
                }
                let bob = (time * 1.5 + k as f32).sin() * 0.12;
                let by = 0.9 + bob;
                let hull = [WHITE, Color::new(0.2, 0.45, 0.75, 1.0), Color::new(0.85, 0.25, 0.2, 1.0), Color::new(0.95, 0.8, 0.3, 1.0)][((hash(k, 62) * 4.0) as usize).min(3)];
                let len = 2.2 + hash(k, 63) * 1.6;
                rect(v, bx - len, by, bx + len, by + 0.8, darker(hull, dk));
                tri(v, vec2(bx + len, by), vec2(bx + len, by + 0.8), vec2(bx + len + 1.0, by + 0.8), darker(hull, dk));
                rect(v, bx - len, by + 0.6, bx + len + 0.8, by + 0.8, darker(Color::new(0.2, 0.25, 0.35, 1.0), dk));
                if hash(k, 64) < 0.7 {
                    let mh = 6.0 + hash(k, 65) * 5.0;
                    line(v, vec2(bx, by + 0.8), vec2(bx, by + mh), 0.12, darker(Color::new(0.9, 0.9, 0.88, 1.0), dk));
                    line(v, vec2(bx, by + 1.6), vec2(bx - len * 0.9, by + 1.4), 0.1, darker(Color::new(0.9, 0.9, 0.88, 1.0), dk));
                    if hash(k, 66) < 0.5 {
                        tri(v, vec2(bx + 0.15, by + 1.8), vec2(bx + 0.15, by + mh - 0.3), vec2(bx + len * 0.9, by + 1.8), darker(Color::new(0.98, 0.97, 0.93, 1.0), dk));
                    }
                } else {
                    rect(v, bx - 0.6, by + 0.8, bx + 0.7, by + 1.7, darker(WHITE, dk * 0.95));
                }
            }
        }
        Zone::Canebiere => {
            // L'eglise des Reformes, en haut de la Canebiere
            let pale = darker(Color::new(0.93, 0.9, 0.82, 1.0), dk);
            let pale2 = darker(Color::new(0.82, 0.79, 0.7, 1.0), dk);
            rect(v, x - 5.0, 0.0, x + 5.0, 8.5, pale);
            tri(v, vec2(x - 5.0, 8.5), vec2(x + 5.0, 8.5), vec2(x, 11.5), pale2);
            for sx in [-4.0, 4.0] {
                rect(v, x + sx - 1.3, 0.0, x + sx + 1.3, 11.5, pale2);
                rect(v, x + sx - 0.4, 8.0, x + sx + 0.4, 10.6, window_col(night, false));
                tri(v, vec2(x + sx - 1.3, 11.5), vec2(x + sx + 1.3, 11.5), vec2(x + sx, 16.5), pale);
            }
            circ(v, vec2(x, 6.4), 1.3, window_col(night, true));
            for k in -1..=1 {
                half_ellipse(v, x + k as f32 * 2.4, 0.0, 0.9, 3.4, darker(INK, 1.5));
            }
        }
        Zone::Panier => {
            // La Vieille Charite : arcades de pierre rose et coupole ovale
            let rose = darker(Color::new(0.93, 0.78, 0.68, 1.0), dk);
            let rose2 = darker(Color::new(0.82, 0.66, 0.56, 1.0), dk);
            rect(v, x - 10.0, 0.0, x + 10.0, 8.4, rose);
            for f in 0..3 {
                for k in 0..9 {
                    let ax = x - 9.0 + k as f32 * 2.25;
                    let ay = 0.3 + f as f32 * 2.7;
                    rect(v, ax - 0.7, ay, ax + 0.7, ay + 1.4, darker(INK, 1.6));
                    half_ellipse(v, ax, ay + 1.4, 0.7, 0.7, darker(INK, 1.6));
                }
                rect(v, x - 10.1, 2.6 + f as f32 * 2.7, x + 10.1, 2.85 + f as f32 * 2.7, rose2);
            }
            rect(v, x - 10.3, 8.4, x + 10.3, 9.0, darker(TILE, dk));
            // la chapelle au milieu
            rect(v, x - 3.2, 0.0, x + 3.2, 10.5, rose2);
            for k in 0..4 {
                rect(v, x - 2.6 + k as f32 * 1.6, 0.0, x - 2.2 + k as f32 * 1.6, 9.0, rose);
            }
            tri(v, vec2(x - 3.5, 9.0), vec2(x + 3.5, 9.0), vec2(x, 11.2), rose);
            rect(v, x - 2.6, 10.5, x + 2.6, 12.5, rose2);
            half_ellipse(v, x, 12.5, 2.8, 3.6, darker(Color::new(0.75, 0.72, 0.66, 1.0), dk));
            rect(v, x - 0.25, 16.0, x + 0.25, 17.4, rose2);
        }
        Zone::BonneMere => {
            // Notre-Dame de la Garde sur sa colline, la Bonne Mere tout en haut
            let rock = darker(LIMESTONE, dk);
            let green = darker(Color::new(0.42, 0.55, 0.32, 1.0), dk);
            half_ellipse(v, x, 0.0, 15.0, 6.6, rock);
            half_ellipse(v, x - 5.0, 0.0, 8.0, 4.0, green);
            half_ellipse(v, x + 7.0, 0.0, 6.0, 3.0, green);
            for k in 0..7 {
                let hx = x - 9.0 + k as f32 * 3.0;
                let hy = 0.6 + hash(k, 71) * 2.2;
                rect(v, hx, hy, hx + 1.6, hy + 1.3, darker(Color::new(0.95, 0.85, 0.65, 1.0), dk));
                tri(v, vec2(hx - 0.2, hy + 1.3), vec2(hx + 1.8, hy + 1.3), vec2(hx + 0.8, hy + 2.0), darker(TILE, dk));
            }
            // la basilique, rayee blanc et vert
            let up = lowered(v, 4.2);
            let v = &up;
            let white = darker(Color::new(0.96, 0.94, 0.88, 1.0), dk);
            let stripe = darker(Color::new(0.45, 0.55, 0.45, 1.0), dk);
            rect(v, x - 4.5, 10.0, x + 3.0, 14.0, white);
            for k in 0..4 {
                rect(v, x - 4.5, 10.5 + k as f32 * 0.9, x + 3.0, 10.8 + k as f32 * 0.9, stripe);
            }
            half_ellipse(v, x - 2.6, 14.0, 1.7, 1.9, stripe);
            rect(v, x + 1.0, 10.0, x + 3.6, 20.0, white);
            for k in 0..9 {
                rect(v, x + 1.0, 10.6 + k as f32 * 1.05, x + 3.6, 10.9 + k as f32 * 1.05, stripe);
            }
            rect(v, x + 1.9, 16.2, x + 2.7, 18.2, darker(INK, 1.6));
            rect(v, x + 1.5, 20.0, x + 3.1, 21.6, white);
            // la Vierge doree
            let gold = mix(darker(GOLD, dk), Color::new(1.0, 0.9, 0.4, 1.0), night * 0.5);
            ellipse(v, vec2(x + 2.3, 23.0), 0.5, 1.5, 0.0, gold);
            circ(v, vec2(x + 2.3, 24.7), 0.42, gold);
            circ(v, vec2(x + 1.7, 23.6), 0.3, gold); // l'enfant
        }
        Zone::Velodrome => {
            // Le Velodrome et son grand toit blanc tout en vagues
            let white = darker(Color::new(0.96, 0.96, 0.97, 1.0), dk);
            let grey = darker(Color::new(0.8, 0.82, 0.86, 1.0), dk);
            let om = darker(Color::new(0.2, 0.62, 0.9, 1.0), dk);
            rect(v, x - 15.0, 0.0, x + 15.0, 6.0, darker(Color::new(0.6, 0.62, 0.66, 1.0), dk));
            for k in 0..15 {
                rect(v, x - 14.5 + k as f32 * 2.0, 0.6, x - 13.2 + k as f32 * 2.0, 5.0, if k % 2 == 0 { om } else { white });
            }
            half_ellipse(v, x - 8.0, 6.0, 9.5, 8.5, grey);
            half_ellipse(v, x + 8.0, 6.0, 9.5, 8.5, grey);
            half_ellipse(v, x, 6.0, 12.0, 6.0, white);
            half_ellipse(v, x - 9.0, 6.0, 8.0, 7.2, white);
            half_ellipse(v, x + 9.0, 6.0, 8.0, 7.2, white);
            for k in -8..=8 {
                let lx = x + k as f32 * 2.0;
                line(v, vec2(lx, 6.0), vec2(lx + k as f32 * 0.25, 10.2 - (k as f32).abs() * 0.1), 0.06, grey);
            }
            rect(v, x - 17.0, 5.6, x + 17.0, 6.2, grey);
            let sp = v.p(vec2(x, 3.0));
            comic_text("DROIT AU BUT", sp.x, sp.y, (v.l(1.5) as u16).max(10), WHITE, darker(om, 0.6));
            if night > 0.3 {
                for sx in [-16.0, 16.0] {
                    circ(v, vec2(x + sx, 13.0), 0.9, Color::new(1.0, 1.0, 0.85, night));
                }
            }
        }
        Zone::Corniche => {
            // Au large : le chateau d'If sur son ile, et quelques voiles
            let rock = darker(LIMESTONE, dk);
            let fort = darker(Color::new(0.88, 0.8, 0.66, 1.0), dk);
            half_ellipse(v, x, 1.2, 13.0, 2.6, rock);
            half_ellipse(v, x + 16.0, 1.2, 6.0, 1.3, darker(rock, 0.93));
            rect(v, x - 5.0, 3.2, x + 5.0, 6.2, fort);
            for sx in [-5.0, 0.0, 5.0f32] {
                rect(v, x + sx - 1.3, 3.0, x + sx + 1.3, 7.4, darker(fort, 0.93));
                rect(v, x + sx - 1.5, 7.4, x + sx + 1.5, 7.8, fort);
            }
            rect(v, x - 0.3, 4.6, x + 0.3, 5.6, darker(INK, 1.6));
            for k in [-4i32, -3, 3, 4, 5] {
                let bx = x + k as f32 * 9.0 + hash(k, 81) * 4.0;
                let by = 1.4 + (time * 1.3 + k as f32).sin() * 0.1;
                rect(v, bx - 1.2, by, bx + 1.2, by + 0.4, darker(WHITE, dk));
                tri(v, vec2(bx, by + 0.5), vec2(bx, by + 4.0), vec2(bx + 1.6, by + 0.5), darker(WHITE, dk));
                tri(v, vec2(bx - 0.15, by + 0.5), vec2(bx - 0.15, by + 3.2), vec2(bx - 1.2, by + 0.5), darker(Color::new(0.95, 0.4, 0.3, 1.0), dk));
            }
        }
        Zone::Bourguet => {
            // La cathedrale Notre-Dame-du-Bourguet et son clocher carre
            let st = darker(Color::new(0.86, 0.8, 0.68, 1.0), dk);
            let st2 = darker(Color::new(0.76, 0.7, 0.58, 1.0), dk);
            rect(v, x - 8.0, 0.0, x + 4.0, 8.6, st);
            tri(v, vec2(x - 8.3, 8.6), vec2(x + 4.3, 8.6), vec2(x - 2.0, 11.2), darker(TILE, dk));
            for k in 0..3 {
                let wx = x - 6.5 + k as f32 * 3.0;
                rect(v, wx, 3.4, wx + 0.9, 6.0, window_col(night, k == 1));
                half_ellipse(v, wx + 0.45, 6.0, 0.45, 0.7, window_col(night, k == 1));
            }
            half_ellipse(v, x - 2.0, 0.0, 1.4, 3.6, darker(INK, 1.5));
            circ(v, vec2(x - 2.0, 7.4), 0.9, window_col(night, true));
            // le clocher
            rect(v, x + 4.0, 0.0, x + 8.0, 12.6, st2);
            for k in 0..2 {
                rect(v, x + 4.7 + k as f32 * 1.6, 9.4, x + 5.7 + k as f32 * 1.6, 11.8, darker(INK, 1.5));
            }
            rect(v, x + 3.8, 12.6, x + 8.2, 13.1, st);
            tri(v, vec2(x + 4.0, 13.1), vec2(x + 8.0, 13.1), vec2(x + 6.0, 16.0), st);
            // un etal du marche et son parasol
            for k in [-13.0f32, -10.5] {
                rect(v, x + k, 0.0, x + k + 1.8, 1.0, darker(Color::new(0.55, 0.4, 0.25, 1.0), dk));
                tri(v, vec2(x + k - 0.5, 2.4), vec2(x + k + 2.3, 2.4), vec2(x + k + 0.9, 3.4), darker(if k < -12.0 { Color::new(0.9, 0.3, 0.25, 1.0) } else { Color::new(0.95, 0.8, 0.3, 1.0) }, dk));
                line(v, vec2(x + k + 0.9, 1.0), vec2(x + k + 0.9, 2.5), 0.08, INK);
            }
        }
        Zone::Citadelle => {
            // La colline de la Citadelle et la chapelle Notre-Dame-de-Provence
            let green = darker(Color::new(0.45, 0.56, 0.32, 1.0), dk);
            half_ellipse(v, x, 0.0, 14.0, 6.8, green);
            half_ellipse(v, x - 6.0, 0.0, 7.0, 4.0, darker(green, 0.9));
            for k in 0..8 {
                // cypres et pins
                let cx = x - 11.0 + k as f32 * 3.1;
                let cy = 1.5 + hash(k, 72) * 2.6;
                tri(v, vec2(cx - 0.4, cy), vec2(cx + 0.4, cy), vec2(cx, cy + 3.0), darker(Color::new(0.18, 0.33, 0.2, 1.0), dk));
            }
            for k in 0..5 {
                let hx = x - 8.0 + k as f32 * 3.6;
                let hy = 0.6 + hash(k, 71) * 2.6;
                rect(v, hx, hy, hx + 1.9, hy + 1.5, darker(Color::new(0.9, 0.83, 0.68, 1.0), dk));
                tri(v, vec2(hx - 0.2, hy + 1.5), vec2(hx + 2.1, hy + 1.5), vec2(hx + 0.95, hy + 2.2), darker(TILE, dk));
            }
            // la chapelle octogonale, sa coupole et sa Vierge
            let up = lowered(v, 4.2);
            let v = &up;
            let st = darker(Color::new(0.92, 0.88, 0.78, 1.0), dk);
            rect(v, x - 2.6, 10.6, x + 2.6, 14.5, st);
            for k in 0..3 {
                let wx = x - 1.9 + k as f32 * 1.5;
                rect(v, wx, 11.6, wx + 0.8, 13.4, darker(INK, 1.6));
                half_ellipse(v, wx + 0.4, 13.4, 0.4, 0.45, darker(INK, 1.6));
            }
            rect(v, x - 2.9, 14.5, x + 2.9, 14.9, darker(st, 0.9));
            half_ellipse(v, x, 14.9, 2.5, 2.4, darker(Color::new(0.7, 0.72, 0.74, 1.0), dk));
            rect(v, x - 0.3, 17.2, x + 0.3, 18.2, st);
            ellipse(v, vec2(x, 19.2), 0.35, 1.0, 0.0, darker(Color::new(0.97, 0.97, 0.95, 1.0), dk));
            circ(v, vec2(x, 20.4), 0.3, darker(Color::new(0.97, 0.97, 0.95, 1.0), dk));
        }
        Zone::Lavande => {
            // Un borie, des ruches et des cypres au milieu de la lavande
            let st = darker(Color::new(0.8, 0.77, 0.68, 1.0), dk);
            half_ellipse(v, x, 1.2, 3.2, 4.4, st);
            for k in 0..4 {
                line(v, vec2(x - 2.9 + k as f32 * 0.3, 1.9 + k as f32 * 0.9), vec2(x + 2.9 - k as f32 * 0.3, 1.9 + k as f32 * 0.9), 0.07, darker(st, 0.82));
            }
            rect(v, x - 0.6, 1.2, x + 0.6, 3.0, darker(INK, 1.6));
            for k in [-34.0f32, -22.0, -14.0, 12.0, 20.0, 31.0] {
                let ch = 6.0 + hash(k as i32, 73) * 4.0;
                rect(v, x + k - 0.15, 1.2, x + k + 0.15, 2.0, darker(Color::new(0.4, 0.3, 0.2, 1.0), dk));
                tri(v, vec2(x + k - 0.8, 1.8), vec2(x + k + 0.8, 1.8), vec2(x + k, 1.8 + ch), darker(Color::new(0.18, 0.33, 0.2, 1.0), dk));
            }
            for k in 0..5 {
                let rx = x + 5.5 + k as f32 * 1.1;
                rect(v, rx, 1.2, rx + 0.8, 2.1, darker([Color::new(0.95, 0.85, 0.4, 1.0), Color::new(0.5, 0.7, 0.9, 1.0), WHITE][(k % 3) as usize], dk));
                rect(v, rx - 0.08, 2.1, rx + 0.88, 2.25, darker(Color::new(0.5, 0.4, 0.3, 1.0), dk));
            }
            for k in [-28.0f32, -9.0, 25.0, 37.0] {
                draw_olive(v, x + k, 1.2, 4.2 + hash(k as i32, 83) * 1.2, dk);
            }
            // les rangs de lavande, au premier plan du champ
            draw_lavender_rows(v, x, 92.0, dk);
        }
        Zone::Oliviers => {
            // L'oliveraie : un mas, une restanque en pierre seche, des oliviers en rangs
            let st = darker(Color::new(0.88, 0.8, 0.64, 1.0), dk);
            rect(v, x - 4.5, 1.6, x + 4.5, 6.4, st);
            tri(v, vec2(x - 5.0, 6.4), vec2(x + 5.0, 6.4), vec2(x, 8.2), darker(TILE, dk));
            rect(v, x + 4.5, 1.6, x + 8.0, 4.6, darker(st, 0.92));
            tri(v, vec2(x + 4.5, 4.6), vec2(x + 8.3, 4.6), vec2(x + 4.5, 5.6), darker(TILE, dk * 0.9));
            for k in 0..3 {
                let wx = x - 3.4 + k as f32 * 2.8;
                rect(v, wx, 4.0, wx + 1.0, 5.6, window_col(night, k == 2));
                rect(v, wx - 0.45, 4.0, wx - 0.03, 5.6, darker(Color::new(0.6, 0.55, 0.8, 1.0), dk));
                rect(v, wx + 1.03, 4.0, wx + 1.45, 5.6, darker(Color::new(0.6, 0.55, 0.8, 1.0), dk));
            }
            rect(v, x - 0.7, 1.6, x + 0.7, 3.6, darker(Color::new(0.45, 0.4, 0.6, 1.0), dk));
            // restanque
            for k in -24..=24 {
                let wx = x + k as f32 * 1.6;
                rect(v, wx, 1.0, wx + 1.55, 1.6, darker(if k % 2 == 0 { Color::new(0.8, 0.77, 0.68, 1.0) } else { Color::new(0.72, 0.69, 0.6, 1.0) }, dk));
            }
            for k in -6..=6 {
                if k == 0 || k == 1 {
                    continue; // le mas est la
                }
                let ox = x + k as f32 * 6.2 + hash(k, 84) * 1.5;
                draw_olive(v, ox, 1.6, 4.6 + hash(k, 85) * 1.6, dk);
            }
            for k in -7..=7 {
                let ox = x + k as f32 * 5.6 + 2.5;
                draw_olive(v, ox, 0.0, 3.6 + hash(k, 86) * 1.2, dk);
            }
        }
        Zone::Mourres => {
            // Les Mourres : des champignons de calcaire
            let rock = darker(LIMESTONE, dk);
            let rock2 = darker(Color::new(0.78, 0.75, 0.68, 1.0), dk);
            for k in -5..=5 {
                let rx = x + k as f32 * 7.0 + hash(k, 75) * 3.0;
                let hh = 2.5 + hash(k, 76) * 4.5;
                let wd = 1.0 + hash(k, 77) * 0.8;
                // le pied, puis le chapeau
                tri(v, vec2(rx - wd, 0.9), vec2(rx + wd, 0.9), vec2(rx, 0.9 + hh * 1.3), rock2);
                rect(v, rx - wd * 0.55, 0.9, rx + wd * 0.55, 0.9 + hh, rock2);
                ellipse(v, vec2(rx, 0.9 + hh), wd * 1.9, 0.85, 0.0, rock);
                half_ellipse(v, rx, 0.9 + hh, wd * 1.9, 1.3, rock);
                ellipse(v, vec2(rx, 0.75 + hh), wd * 1.5, 0.35, 0.0, darker(rock2, 0.85));
                if hash(k, 78) < 0.5 {
                    half_ellipse(v, rx + 3.2, 0.9, 1.4, 1.0, darker(Color::new(0.4, 0.5, 0.3, 1.0), dk)); // un chene kermes
                } else {
                    draw_olive(v, rx + 3.4, 0.6, 3.4 + hash(k, 85), dk);
                }
            }
            draw_lavender_rows(v, x - 24.0, 20.0, dk);
            draw_lavender_rows(v, x + 22.0, 24.0, dk);
        }
        Zone::GrandePoste => {
            // La Grande Poste d'Alger : trois grands arcs, deux tourelles a coupole
            let white = darker(Color::new(0.98, 0.98, 0.96, 1.0), dk);
            let shade2 = darker(Color::new(0.88, 0.88, 0.86, 1.0), dk);
            let blue = darker(Color::new(0.15, 0.4, 0.7, 1.0), dk);
            rect(v, x - 10.0, 0.0, x + 10.0, 11.0, white);
            rect(v, x - 10.2, 11.0, x + 10.2, 11.6, shade2);
            for k in -1..=1 {
                let ax = x + k as f32 * 4.6;
                rect(v, ax - 1.5, 1.2, ax + 1.5, 6.0, darker(INK, 1.6));
                circ(v, vec2(ax, 6.2), 1.75, darker(INK, 1.6)); // arc outrepasse
                rect(v, ax - 1.5, 1.2, ax + 1.5, 4.4, window_col(night, true));
                rect(v, ax - 2.0, 8.4, ax + 2.0, 8.8, blue); // frise de faience
            }
            rect(v, x - 8.0, 0.0, x + 8.0, 1.2, shade2); // les marches
            for k in 0..9 {
                rect(v, x - 9.5 + k as f32 * 2.2, 11.6, x - 8.6 + k as f32 * 2.2, 12.4, white); // merlons
            }
            for sx in [-9.0, 9.0] {
                rect(v, x + sx - 1.6, 0.0, x + sx + 1.6, 14.0, shade2);
                rect(v, x + sx - 0.5, 9.5, x + sx + 0.5, 11.5, darker(INK, 1.6));
                half_ellipse(v, x + sx, 14.0, 1.7, 2.2, white);
                line(v, vec2(x + sx, 16.2), vec2(x + sx, 17.4), 0.12, darker(GOLD, dk));
            }
            for sx in [-14.0f32, 14.0] {
                draw_palm(v, x + sx, 0.0, 7.0, dk, time);
            }
        }
        Zone::Casbah => {
            // La mosquee Ketchaoua : grand escalier, trois arcs, deux minarets
            let white = darker(Color::new(0.97, 0.96, 0.92, 1.0), dk);
            let shade2 = darker(Color::new(0.86, 0.84, 0.78, 1.0), dk);
            let blue = darker(Color::new(0.15, 0.45, 0.7, 1.0), dk);
            for k in 0..4 {
                rect(v, x - 7.0 + k as f32 * 0.5, k as f32 * 0.5, x + 7.0 - k as f32 * 0.5, k as f32 * 0.5 + 0.5, shade2);
            }
            rect(v, x - 5.0, 2.0, x + 5.0, 10.5, white);
            for k in -1..=1 {
                let ax = x + k as f32 * 3.0;
                rect(v, ax - 1.0, 2.0, ax + 1.0, 5.6, darker(INK, 1.6));
                circ(v, vec2(ax, 5.8), 1.15, darker(INK, 1.6));
            }
            rect(v, x - 5.0, 8.2, x + 5.0, 8.6, blue);
            half_ellipse(v, x, 10.5, 2.6, 2.6, shade2);
            for sx in [-6.5, 6.5] {
                rect(v, x + sx - 1.2, 0.0, x + sx + 1.2, 11.6, white);
                for k in 0..3 {
                    rect(v, x + sx - 1.2, 3.0 + k as f32 * 3.0, x + sx + 1.2, 3.3 + k as f32 * 3.0, blue);
                }
                rect(v, x + sx - 1.5, 11.6, x + sx + 1.5, 12.1, shade2);
                rect(v, x + sx - 0.8, 12.1, x + sx + 0.8, 13.6, white);
                half_ellipse(v, x + sx, 13.6, 0.9, 1.0, shade2);
                line(v, vec2(x + sx, 14.6), vec2(x + sx, 15.5), 0.1, darker(GOLD, dk));
            }
        }
        Zone::Maqam => {
            // Maqam Echahid : trois palmes de beton qui se rejoignent, face a la baie
            let conc = darker(Color::new(0.88, 0.87, 0.84, 1.0), dk);
            let conc2 = darker(Color::new(0.76, 0.75, 0.72, 1.0), dk);
            half_ellipse(v, x, 0.9, 16.0, 3.0, darker(Color::new(0.45, 0.58, 0.36, 1.0), dk));
            for (sx, col) in [(-4.6f32, conc2), (4.6, conc2), (0.0, conc)] {
                // chaque palme : large au pied, effilee et courbee vers le sommet
                let mut prev = vec2(x + sx, 2.6);
                for k in 1..=10 {
                    let t = k as f32 / 10.0;
                    let p = vec2(x + sx * (1.0 - t).powf(1.7), 2.6 + t * 9.4);
                    seg(v, prev, p, 1.9 * (1.0 - t * 0.78), col);
                    prev = p;
                }
            }
            rect(v, x - 0.8, 11.6, x + 0.8, 13.5, conc);
            half_ellipse(v, x, 13.5, 0.9, 0.8, conc2);
            line(v, vec2(x, 14.3), vec2(x, 15.5), 0.12, conc2);
            circ(v, vec2(x, 4.0), 0.45, Color::new(1.0, 0.6, 0.15, 0.6 + 0.4 * night)); // la flamme
            for sx in [-22.0f32, -13.0, 13.0, 21.0, 30.0] {
                draw_palm(v, x + sx, 0.9, 5.5 + hash(sx as i32, 79) * 2.5, dk, time);
            }
        }
        Zone::NotreDameAfrique => {
            // Notre-Dame d'Afrique sur son promontoire
            let rock = darker(Color::new(0.8, 0.74, 0.6, 1.0), dk);
            let green = darker(Color::new(0.42, 0.55, 0.32, 1.0), dk);
            half_ellipse(v, x, 0.0, 14.0, 6.2, rock);
            half_ellipse(v, x + 6.0, 0.0, 7.0, 3.5, green);
            half_ellipse(v, x - 7.0, 0.0, 6.0, 3.0, green);
            for k in 0..6 {
                let hx = x - 9.0 + k as f32 * 3.4;
                let hy = 0.6 + hash(k, 71) * 2.2;
                rect(v, hx, hy, hx + 1.8, hy + 1.5, darker(Color::new(0.98, 0.98, 0.96, 1.0), dk));
            }
            let ochre = darker(Color::new(0.93, 0.85, 0.68, 1.0), dk);
            let blue = darker(Color::new(0.2, 0.45, 0.7, 1.0), dk);
            let up = lowered(v, 3.8);
            let v = &up;
            rect(v, x - 4.0, 9.4, x + 3.0, 13.4, ochre);
            rect(v, x - 4.0, 12.4, x + 3.0, 12.8, blue); // frise de faience bleue
            for k in 0..4 {
                let wx = x - 3.4 + k as f32 * 1.6;
                rect(v, wx, 10.2, wx + 0.7, 11.8, darker(INK, 1.6));
                half_ellipse(v, wx + 0.35, 11.8, 0.35, 0.4, darker(INK, 1.6));
            }
            // la grande coupole et le clocher
            rect(v, x - 2.6, 13.4, x + 0.6, 15.0, ochre);
            half_ellipse(v, x - 1.0, 15.0, 2.0, 2.8, darker(Color::new(0.55, 0.62, 0.66, 1.0), dk));
            line(v, vec2(x - 1.0, 17.8), vec2(x - 1.0, 19.2), 0.12, darker(GOLD, dk));
            rect(v, x + 2.2, 9.4, x + 4.2, 18.0, ochre);
            rect(v, x + 2.2, 15.4, x + 4.2, 15.8, blue);
            rect(v, x + 2.8, 16.2, x + 3.6, 17.5, darker(INK, 1.6));
            half_ellipse(v, x + 3.2, 18.0, 1.1, 1.5, darker(Color::new(0.55, 0.62, 0.66, 1.0), dk));
            line(v, vec2(x + 3.2, 19.5), vec2(x + 3.2, 20.6), 0.1, darker(GOLD, dk));
        }
        Zone::Sahara => {
            // Le Sahara : grandes dunes, une oasis et une caravane
            let sand = darker(SAND, dk);
            let sand2 = darker(Color::new(0.86, 0.68, 0.4, 1.0), dk);
            for k in -4..=4 {
                let dx = x + k as f32 * 11.0;
                let dh = 4.0 + hash(k, 82) * 4.0;
                half_ellipse(v, dx, 0.0, 9.0, dh, sand);
                half_ellipse(v, dx + 4.0, 0.0, 6.0, dh * 0.7, sand2);
            }
            for sx in [-3.0f32, 0.0, 2.5, 5.0] {
                draw_palm(v, x + sx, 0.4, 5.0 + hash(sx as i32, 79) * 2.5, dk, time);
            }
            ellipse(v, vec2(x + 1.0, 0.5), 5.5, 0.5, 0.0, darker(Color::new(0.2, 0.55, 0.7, 1.0), dk));
            // la caravane, sur la crete
            let camel = darker(Color::new(0.45, 0.32, 0.2, 1.0), dk);
            for k in 0..4 {
                let cx = x - 28.0 + k as f32 * 3.4;
                let cy = 5.2 + (k as f32 * 1.3).sin() * 0.3;
                ellipse(v, vec2(cx, cy + 1.3), 1.0, 0.5, 0.0, camel);
                half_ellipse(v, cx - 0.1, cy + 1.6, 0.45, 0.6, camel); // la bosse
                line(v, vec2(cx + 0.8, cy + 1.4), vec2(cx + 1.3, cy + 2.3), 0.22, camel);
                circ(v, vec2(cx + 1.45, cy + 2.35), 0.22, camel);
                for lx in [-0.6f32, -0.3, 0.4, 0.7] {
                    let sw = (time * 3.0 + lx * 4.0 + k as f32).sin() * 0.12;
                    line(v, vec2(cx + lx, cy + 1.1), vec2(cx + lx + sw, cy), 0.12, camel);
                }
            }
        }
    }
}

/// La meme vue, decalee de `d` metres vers le bas (pour poser un monument
/// au sommet d'une colline sans qu'il sorte de l'ecran).
fn lowered(v: &View, d: f32) -> View {
    View { cam_x: v.cam_x, cam_y: v.cam_y, s: v.s, ground_sy: v.ground_sy + d * v.s, anchor_x: v.anchor_x, shake: v.shake }
}

/// Un olivier : tronc noueux, feuillage vert argente.
fn draw_olive(v: &View, x: f32, y: f32, h: f32, dk: f32) {
    let trunk = darker(Color::new(0.42, 0.36, 0.3, 1.0), dk);
    let leaf = darker(Color::new(0.52, 0.62, 0.45, 1.0), dk);
    let leaf2 = darker(Color::new(0.66, 0.74, 0.6, 1.0), dk);
    seg(v, vec2(x - 0.1, y), vec2(x + 0.15, y + h * 0.35), 0.5, trunk);
    seg(v, vec2(x + 0.15, y + h * 0.35), vec2(x - 0.5, y + h * 0.62), 0.3, trunk);
    seg(v, vec2(x + 0.15, y + h * 0.35), vec2(x + 0.7, y + h * 0.6), 0.3, trunk);
    for (dx, dy, r) in [(-0.9f32, 0.68f32, 0.95f32), (0.9, 0.66, 1.0), (0.0, 0.85, 1.15), (-0.4, 0.95, 0.7), (0.5, 0.98, 0.7)] {
        ellipse(v, vec2(x + dx * h * 0.32, y + h * dy), r * h * 0.3, r * h * 0.21, 0.0, leaf);
    }
    for (dx, dy) in [(-0.6f32, 0.82f32), (0.3, 0.98), (0.8, 0.76)] {
        ellipse(v, vec2(x + dx * h * 0.32, y + h * dy), h * 0.13, h * 0.07, 0.0, leaf2);
    }
}

/// Un champ de lavande en rangs, vu de face (largeur `w` autour de `x`).
fn draw_lavender_rows(v: &View, x: f32, w: f32, dk: f32) {
    let n = (w / 1.9) as i32;
    for row in (0..3).rev() {
        for k in -n / 2..=n / 2 {
            let lx = x + k as f32 * 1.9 + row as f32 * 0.6;
            let ly = row as f32 * 0.55;
            let col = if (k + row) % 2 == 0 { LAVENDER } else { Color::new(0.48, 0.36, 0.7, 1.0) };
            half_ellipse(v, lx, ly, 0.9, 0.95 + hash(k + row * 31, 74) * 0.3, darker(col, dk * (1.0 - row as f32 * 0.07)));
            half_ellipse(v, lx, ly, 0.9, 0.3, darker(Color::new(0.35, 0.5, 0.3, 1.0), dk));
        }
    }
}

/// Un palmier (les palmes ondulent un peu).
fn draw_palm(v: &View, x: f32, y: f32, h: f32, dk: f32, time: f32) {
    let trunk = darker(Color::new(0.5, 0.38, 0.25, 1.0), dk);
    let leaf = darker(Color::new(0.2, 0.5, 0.25, 1.0), dk);
    let top = vec2(x + 0.4, y + h);
    seg(v, vec2(x, y), vec2(x + 0.25, y + h * 0.5), 0.4, trunk);
    seg(v, vec2(x + 0.25, y + h * 0.5), top, 0.32, trunk);
    for k in 0..7 {
        let a = -0.5 + k as f32 * (PI + 1.0) / 6.0 + (time * 1.2 + x).sin() * 0.05;
        let mid = top + Vec2::from_angle(a) * 1.4;
        let tip = mid + Vec2::from_angle(a - 0.9 * a.cos().signum()) * 1.3 + vec2(0.0, -0.5);
        seg(v, top, mid, 0.22, leaf);
        seg(v, mid, tip, 0.14, leaf);
    }
}

/// Facade du Sud. Marseille : crepi ocre ou rose, volets de couleur, tuiles
/// romaines. Forcalquier : pierre et volets lavande. Alger : murs blancs,
/// volets bleus et toits en terrasse.
fn draw_facade_south(v: &View, i: i32, x0: f32, bw: f32, zone: Zone, night: f32) {
    let dk = 1.0 - 0.6 * night;
    let city = zone.city();
    let floors = if zone == Zone::Casbah { 2 + (hash(i, 2) * 3.0) as i32 } else { 3 + (hash(i, 2) * 2.0) as i32 };
    let fh = 3.0;
    let top = floors as f32 * fh;
    let walls: [Color; 5] = match city {
        City::Forcalquier => [
            Color::new(0.9, 0.84, 0.7, 1.0),
            Color::new(0.84, 0.78, 0.66, 1.0),
            Color::new(0.94, 0.86, 0.62, 1.0),
            Color::new(0.8, 0.76, 0.68, 1.0),
            Color::new(0.92, 0.78, 0.6, 1.0),
        ],
        City::Algerie => [
            Color::new(0.98, 0.98, 0.96, 1.0),
            Color::new(0.94, 0.95, 0.96, 1.0),
            Color::new(0.97, 0.95, 0.9, 1.0),
            Color::new(0.92, 0.93, 0.92, 1.0),
            Color::new(0.99, 0.97, 0.93, 1.0),
        ],
        _ => [
            Color::new(0.95, 0.8, 0.52, 1.0),
            Color::new(0.94, 0.72, 0.62, 1.0),
            Color::new(0.97, 0.89, 0.64, 1.0),
            Color::new(0.96, 0.92, 0.82, 1.0),
            Color::new(0.9, 0.62, 0.45, 1.0),
        ],
    };
    let shutters: [Color; 4] = match city {
        City::Forcalquier => [Color::new(0.6, 0.55, 0.8, 1.0), Color::new(0.5, 0.6, 0.72, 1.0), Color::new(0.55, 0.65, 0.5, 1.0), Color::new(0.7, 0.62, 0.82, 1.0)],
        City::Algerie => [Color::new(0.15, 0.4, 0.75, 1.0), Color::new(0.2, 0.5, 0.8, 1.0), Color::new(0.1, 0.55, 0.6, 1.0), Color::new(0.25, 0.35, 0.7, 1.0)],
        _ => [Color::new(0.3, 0.5, 0.72, 1.0), Color::new(0.35, 0.56, 0.42, 1.0), Color::new(0.55, 0.5, 0.75, 1.0), Color::new(0.6, 0.3, 0.25, 1.0)],
    };
    let wall = darker(walls[((hash(i, 3) * 5.0) as usize).min(4)], dk);
    let shut = darker(shutters[((hash(i, 9) * 4.0) as usize).min(3)], dk);
    rect(v, x0, 0.0, x0 + bw - 0.05, top, wall);
    if city == City::Algerie {
        // toit en terrasse : acrotere, et parfois une petite coupole ou du linge
        rect(v, x0 - 0.1, top, x0 + bw + 0.05, top + 0.45, darker(wall, 0.9));
        if hash(i, 15) < 0.3 {
            half_ellipse(v, x0 + bw * 0.5, top + 0.45, 1.2, 1.1, wall);
        } else if hash(i, 15) < 0.6 {
            rect(v, x0 + 1.0, top + 0.45, x0 + 3.2, top + 1.9, darker(wall, 0.95));
            rect(v, x0 + 1.7, top + 0.45, x0 + 2.4, top + 1.5, shut);
        }
        if zone == Zone::Casbah {
            // rondins de thuya sous l'encorbellement
            for k in 0..7 {
                let rx = x0 + 0.5 + k as f32 * 1.1;
                line(v, vec2(rx, fh - 0.05), vec2(rx - 0.25, fh - 0.75), 0.1, darker(Color::new(0.5, 0.35, 0.22, 1.0), dk));
            }
        }
    } else {
        // toit de tuiles, a faible pente
        let tile = darker(TILE, dk);
        rect(v, x0 - 0.2, top, x0 + bw + 0.15, top + 0.35, tile);
        tri(v, vec2(x0 - 0.2, top + 0.35), vec2(x0 + bw + 0.15, top + 0.35), vec2(x0 + bw * 0.5, top + 1.5), darker(tile, 0.9));
        rect(v, x0 + bw * 0.7, top + 0.6, x0 + bw * 0.7 + 0.5, top + 1.9, darker(wall, 0.85));
    }
    for f in 0..floors {
        for k in 0..3 {
            let wx = x0 + 1.1 + k as f32 * 2.3;
            let wy = f as f32 * fh + 0.6;
            if f == 0 {
                if matches!(zone, Zone::Canebiere | Zone::Velodrome | Zone::Bourguet | Zone::GrandePoste) {
                    // boutique avec store raye
                    rect(v, wx - 0.2, 0.0, wx + 1.4, 2.0, window_col(night, true));
                    let stripe = if zone == Zone::Velodrome { Color::new(0.2, 0.62, 0.9, 1.0) } else { shutters[((hash(i * 3 + k, 5) * 4.0) as usize).min(3)] };
                    for b in 0..4 {
                        let sx = wx - 0.35 + b as f32 * 0.5;
                        rect(v, sx, 2.0, sx + 0.5, 2.5, darker(if b % 2 == 0 { stripe } else { WHITE }, dk));
                    }
                } else {
                    rect(v, wx, 0.0, wx + 1.0, 2.1, darker(shut, 0.8));
                    half_ellipse(v, wx + 0.5, 2.1, 0.5, 0.35, darker(shut, 0.8));
                }
            } else {
                if zone == Zone::Casbah {
                    // petites fenetres a moucharabieh
                    rect(v, wx + 0.2, wy + 0.5, wx + 0.8, wy + 1.4, shut);
                    rect(v, wx + 0.3, wy + 0.6, wx + 0.7, wy + 1.3, window_col(night, hash(i * 11 + f * 3 + k, 7) < 0.35));
                    continue;
                }
                rect(v, wx, wy, wx + 1.0, wy + 1.8, window_col(night, hash(i * 11 + f * 3 + k, 7) < 0.35));
                // volets : ouverts, ou fermes pour la sieste
                if hash(i * 13 + f * 5 + k, 10) < 0.25 {
                    rect(v, wx, wy, wx + 1.0, wy + 1.8, shut);
                    line(v, vec2(wx + 0.5, wy), vec2(wx + 0.5, wy + 1.8), 0.04, darker(shut, 0.7));
                } else {
                    rect(v, wx - 0.45, wy, wx - 0.03, wy + 1.8, shut);
                    rect(v, wx + 1.03, wy, wx + 1.45, wy + 1.8, shut);
                }
            }
        }
    }
    // Au Panier, le linge seche aux fenetres
    if zone == Zone::Panier && hash(i, 8) < 0.7 {
        let y = fh * 2.0 + 0.4;
        line(v, vec2(x0 + 0.5, y + 0.3), vec2(x0 + bw - 0.5, y), 0.04, darker(Color::new(0.3, 0.3, 0.3, 1.0), dk));
        let linen = [WHITE, Color::new(0.9, 0.3, 0.3, 1.0), Color::new(0.3, 0.55, 0.85, 1.0), Color::new(0.98, 0.85, 0.3, 1.0)];
        for b in 0..6 {
            let t = (b as f32 + 0.5) / 6.5;
            let lx = x0 + 0.5 + t * (bw - 1.0);
            let ly = y + 0.3 - t * 0.3;
            let c = linen[((hash(i * 7 + b, 12) * 4.0) as usize).min(3)];
            rect(v, lx, ly - 0.9 - hash(i + b, 14) * 0.4, lx + 0.55, ly, darker(c, dk));
        }
    }
    // Au Velodrome, les drapeaux de l'OM aux balcons
    if zone == Zone::Velodrome && hash(i, 8) < 0.6 {
        let y = fh + 0.2;
        rect(v, x0 + 2.6, y, x0 + 5.2, y + 0.5, darker(Color::new(0.2, 0.62, 0.9, 1.0), dk));
        rect(v, x0 + 2.6, y - 0.5, x0 + 5.2, y, darker(WHITE, dk));
    }
}

fn draw_lamp(v: &View, x: f32, night: f32) {
    let iron = Color::new(0.08, 0.12, 0.1, 1.0);
    rect(v, x - 0.06, 0.0, x + 0.06, 3.4, iron);
    rect(v, x - 0.15, 0.0, x + 0.15, 0.5, iron);
    seg(v, vec2(x, 3.4), vec2(x + 0.35, 3.6), 0.05, iron);
    let lc = if night > 0.3 { Color::new(1.0, 0.9, 0.55, 1.0) } else { Color::new(0.85, 0.88, 0.8, 1.0) };
    tri(v, vec2(x - 0.2, 3.4), vec2(x + 0.2, 3.4), vec2(x, 3.15), iron);
    rect(v, x - 0.16, 3.4, x + 0.16, 3.8, lc);
    tri(v, vec2(x - 0.24, 3.8), vec2(x + 0.24, 3.8), vec2(x, 4.05), iron);
    if night > 0.3 {
        circ(v, vec2(x, 3.6), 0.9, Color::new(1.0, 0.85, 0.4, 0.12 * night));
        ellipse(v, vec2(x, 0.02), 1.4, 0.12, 0.0, Color::new(1.0, 0.85, 0.4, 0.15 * night));
    }
}

fn draw_obstacle(v: &View, o: &Obstacle) {
    if o.smashed {
        // il n'en reste que des miettes
        rect(v, o.x, 0.0, o.x + o.w, 0.08, Color::new(0.3, 0.3, 0.3, 0.6));
        return;
    }
    // Renverse : on pivote autour du coin arriere-droit
    let pivot = vec2(o.x + o.w, 0.0);
    let rot = Vec2::from_angle(-o.knocked);
    let t = |p: Vec2| pivot + rot.rotate(p - pivot);
    match o.kind {
        ObsKind::Barrier => {
            let x0 = o.x;
            let x1 = o.x + o.w;
            for lx in [x0 + 0.15, x1 - 0.15] {
                seg(v, t(vec2(lx, 0.0)), t(vec2(lx, 0.9)), 0.07, Color::new(0.3, 0.3, 0.3, 1.0));
                seg(v, t(vec2(lx - 0.25, 0.02)), t(vec2(lx + 0.25, 0.02)), 0.08, Color::new(0.2, 0.2, 0.2, 1.0));
            }
            // panneau raye rouge et blanc
            let n = 7;
            for k in 0..n {
                let a = x0 + k as f32 * o.w / n as f32;
                let b = a + o.w / n as f32;
                let col = if k % 2 == 0 { Color::new(0.9, 0.1, 0.1, 1.0) } else { WHITE };
                let p = [t(vec2(a, 0.55)), t(vec2(b, 0.55)), t(vec2(b, 1.0)), t(vec2(a, 1.0))];
                draw_triangle(v.p(p[0]), v.p(p[1]), v.p(p[2]), col);
                draw_triangle(v.p(p[0]), v.p(p[2]), v.p(p[3]), col);
            }
            if o.knocked == 0.0 {
                let sp = v.p(vec2(x0 + o.w / 2.0, 0.3));
                let size = (v.s * 0.24) as u16;
                comic_text("TRAVAUX", sp.x, sp.y, size.max(10), YELLOW, INK);
            }
        }
        ObsKind::Bike => {
            let wheel = Color::new(0.15, 0.15, 0.17, 1.0);
            let frame = Color::new(0.7, 0.82, 0.9, 1.0);
            let w1 = vec2(o.x + 0.35, 0.35);
            let w2 = vec2(o.x + o.w - 0.35, 0.35);
            for wc in [w1, w2] {
                let p = v.p(t(wc));
                draw_circle_lines(p.x, p.y, v.l(0.33), v.l(0.06), wheel);
                draw_circle(p.x, p.y, v.l(0.05), wheel);
            }
            let seat = vec2(o.x + 0.6, 0.95);
            let bar = vec2(o.x + o.w - 0.5, 1.0);
            let crank = vec2(o.x + 0.8, 0.35);
            seg(v, t(w1), t(crank), 0.07, frame);
            seg(v, t(crank), t(seat), 0.07, frame);
            seg(v, t(seat), t(bar), 0.07, frame);
            seg(v, t(bar), t(w2), 0.07, frame);
            seg(v, t(crank), t(bar), 0.07, frame);
            seg(v, t(seat + vec2(-0.15, 0.05)), t(seat + vec2(0.15, 0.05)), 0.08, INK);
            // panier
            let bk = vec2(o.x + o.w - 0.35, 1.0);
            let pb = [t(bk), t(bk + vec2(0.4, 0.0)), t(bk + vec2(0.35, -0.25)), t(bk + vec2(0.05, -0.25))];
            draw_triangle(v.p(pb[0]), v.p(pb[1]), v.p(pb[2]), frame);
            draw_triangle(v.p(pb[0]), v.p(pb[2]), v.p(pb[3]), frame);
        }
        ObsKind::Bin => {
            let green = Color::new(0.2, 0.45, 0.25, 1.0);
            let dark = Color::new(0.13, 0.3, 0.17, 1.0);
            let x0 = o.x;
            let x1 = o.x + o.w;
            let body = [t(vec2(x0 + 0.05, 0.1)), t(vec2(x1 - 0.05, 0.1)), t(vec2(x1, 1.0)), t(vec2(x0, 1.0))];
            draw_triangle(v.p(body[0]), v.p(body[1]), v.p(body[2]), green);
            draw_triangle(v.p(body[0]), v.p(body[2]), v.p(body[3]), green);
            let lid = [t(vec2(x0 - 0.05, 1.0)), t(vec2(x1 + 0.05, 1.0)), t(vec2(x1 + 0.05, 1.15)), t(vec2(x0 - 0.05, 1.15))];
            draw_triangle(v.p(lid[0]), v.p(lid[1]), v.p(lid[2]), dark);
            draw_triangle(v.p(lid[0]), v.p(lid[2]), v.p(lid[3]), dark);
            for lx in [x0 + 0.12, x1 - 0.12] {
                let p = v.p(t(vec2(lx, 0.07)));
                draw_circle(p.x, p.y, v.l(0.08), INK);
            }
            seg(v, t(vec2(x0 + 0.2, 0.3)), t(vec2(x0 + 0.2, 0.85)), 0.04, dark);
            seg(v, t(vec2(x1 - 0.2, 0.3)), t(vec2(x1 - 0.2, 0.85)), 0.04, dark);
        }
    }
}

fn draw_hazard(v: &View, h: &Hazard, time: f32) {
    let x = h.x;
    let steel_d = Color::new(0.32, 0.33, 0.36, 1.0);
    match h.kind {
        HazKind::Mine => {
            if h.fired {
                // le cratere
                ellipse(v, vec2(x, 0.0), 0.8, 0.12, 0.0, Color::new(0.12, 0.1, 0.08, 1.0));
                if h.t < 1.5 {
                    let a = 0.5 * (1.0 - h.t / 1.5);
                    circ(v, vec2(x, 0.5 + h.t), 0.3 + h.t * 0.4, Color::new(0.3, 0.3, 0.3, a));
                }
                return;
            }
            let olive = Color::new(0.3, 0.36, 0.22, 1.0);
            ellipse(v, vec2(x, 0.03), 0.34, 0.05, 0.0, darker(olive, 0.7));
            half_ellipse(v, x, 0.03, 0.3, 0.18, olive);
            line(v, vec2(x - 0.28, 0.08), vec2(x + 0.28, 0.08), 0.02, darker(olive, 0.6));
            rect(v, x - 0.05, 0.18, x + 0.05, 0.26, steel_d);
            let blink = (time * 4.0).sin() > 0.3;
            circ(v, vec2(x, 0.28), 0.04, if blink { Color::new(1.0, 0.1, 0.1, 1.0) } else { Color::new(0.4, 0.05, 0.05, 1.0) });
            if blink {
                circ(v, vec2(x, 0.28), 0.12, Color::new(1.0, 0.1, 0.1, 0.25));
            }
        }
    }
}

fn draw_pickup(v: &View, p: &Pickup, time: f32) {
    if p.taken {
        return;
    }
    let c = p.pos + vec2(0.0, (time * 3.0 + p.phase).sin() * 0.08);
    match p.kind {
        PickKind::Kebab => {
            let glow = 0.3 + 0.2 * (time * 8.0).sin();
            ellipse(v, c, 0.75, 0.6, 0.0, Color::new(1.0, 0.45, 0.1, glow));
            draw_kebab(v, c, 1.0);
        }
    }
}

/// Un gros sandwich kebab vu de cote : pain turc coupe en deux, garniture
/// qui deborde, frites qui depassent, papier autour d'un bout.
/// Taille `k` (en metres pour k = 1), centre `c`.
fn draw_kebab(v: &View, c: Vec2, k: f32) {
    let crust = Color::new(0.85, 0.55, 0.22, 1.0);
    let crust_d = Color::new(0.68, 0.4, 0.15, 1.0);
    let crumb = Color::new(0.98, 0.88, 0.65, 1.0);
    let meat = Color::new(0.55, 0.3, 0.14, 1.0);
    let meat_d = Color::new(0.4, 0.19, 0.07, 1.0);
    let salad = Color::new(0.38, 0.78, 0.25, 1.0);
    let fry = Color::new(1.0, 0.85, 0.3, 1.0);
    let p = |x: f32, y: f32| c + vec2(x * k, y * k);

    // frites qui depassent a droite
    for (i, y) in [0.02f32, 0.09, 0.15].into_iter().enumerate() {
        let len = 0.78 + i as f32 * 0.06;
        seg(v, p(0.4, y), p(len, y + 0.05 - i as f32 * 0.04), 0.055 * k, fry);
    }
    // pain du bas
    half_ellipse(v, c.x, c.y, 0.62 * k, -0.24 * k, crust);
    half_ellipse(v, c.x, c.y, 0.56 * k, -0.17 * k, crust_d);
    half_ellipse(v, c.x, c.y + 0.005 * k, 0.56 * k, -0.07 * k, crumb);
    // salade (deborde en bas)
    for i in 0..9 {
        let x = -0.6 + i as f32 * 0.15;
        ellipse(v, p(x, 0.0), 0.1 * k, 0.05 * k, 0.4 * if i % 2 == 0 { 1.0 } else { -1.0 }, salad);
    }
    // viande grillee
    for i in 0..8 {
        let x = -0.5 + i as f32 * 0.143;
        ellipse(v, p(x, 0.08 + (i % 2) as f32 * 0.025), 0.1 * k, 0.065 * k, 0.5 * (i as f32 - 3.5) / 3.5, if i % 2 == 0 { meat } else { meat_d });
    }
    // tomates et oignons
    for x in [-0.32, 0.05, 0.38] {
        half_ellipse(v, c.x + x * k, c.y + 0.13 * k, 0.08 * k, 0.06 * k, Color::new(0.9, 0.15, 0.12, 1.0));
        half_ellipse(v, c.x + x * k, c.y + 0.13 * k, 0.05 * k, 0.035 * k, Color::new(1.0, 0.5, 0.4, 1.0));
    }
    for x in [-0.12, 0.24] {
        ellipse(v, p(x, 0.15), 0.08 * k, 0.025 * k, 0.0, Color::new(0.82, 0.7, 0.95, 1.0));
    }
    // sauce blanche qui coule sur le pain du bas
    for x in [-0.45, -0.18, 0.12, 0.42] {
        ellipse(v, p(x, 0.17), 0.08 * k, 0.03 * k, 0.0, WHITE);
    }
    for (x, l) in [(-0.3, 0.2), (0.2, 0.28)] {
        seg(v, p(x, 0.12), p(x, 0.12 - l), 0.04 * k, WHITE);
        circ(v, p(x, 0.12 - l), 0.03 * k, WHITE);
    }
    // pain du haut, bombe, avec graines de sesame
    half_ellipse(v, c.x, c.y + 0.17 * k, 0.66 * k, 0.24 * k, crust);
    half_ellipse(v, c.x - 0.04 * k, c.y + 0.21 * k, 0.52 * k, 0.15 * k, Color::new(0.92, 0.65, 0.3, 1.0));
    line(v, p(-0.6, 0.18), p(0.6, 0.18), 0.03 * k, crumb);
    for (x, y) in [(-0.2, 0.33), (-0.02, 0.37), (0.16, 0.36), (0.34, 0.31), (0.07, 0.29), (0.48, 0.25), (0.25, 0.25)] {
        ellipse(v, p(x, y), 0.03 * k, 0.016 * k, 0.4, Color::new(1.0, 0.97, 0.88, 1.0));
    }
    // papier autour du bout gauche
    let paper = Color::new(0.97, 0.97, 0.95, 1.0);
    let q = [p(-0.7, -0.22), p(-0.38, -0.25), p(-0.36, 0.43), p(-0.7, 0.38)];
    tri(v, q[0], q[1], q[2], paper);
    tri(v, q[0], q[2], q[3], paper);
    for t in [0.2f32, 0.32] {
        let a = q[0] + (q[3] - q[0]) * t;
        let b = q[1] + (q[2] - q[1]) * t;
        line(v, a, b, 0.035 * k, Color::new(0.85, 0.2, 0.15, 1.0));
    }
}

/// Petite roue a pneu.
fn tyre(v: &View, c: Vec2, r: f32, spin: f32) {
    circ(v, c, r, Color::new(0.1, 0.1, 0.1, 1.0));
    circ(v, c, r * 0.55, Color::new(0.75, 0.75, 0.78, 1.0));
    let d = Vec2::from_angle(spin) * r * 0.5;
    line(v, c - d, c + d, r * 0.18, Color::new(0.3, 0.3, 0.32, 1.0));
    line(v, c - d.perp(), c + d.perp(), r * 0.18, Color::new(0.3, 0.3, 0.32, 1.0));
}

/// Marseille : le petit train des touristes (pas de rails, il roule sur le quai).
fn draw_tourist_train(v: &View, x0: f32, night: f32, time: f32) {
    let dk = 1.0 - 0.4 * night;
    let white = darker(Color::new(0.97, 0.97, 0.95, 1.0), dk);
    let blue = darker(Color::new(0.15, 0.5, 0.85, 1.0), dk);
    let navy = darker(Color::new(0.1, 0.25, 0.5, 1.0), dk);
    let spin = time * 6.0;
    // la locomotive, deguisee en loco a vapeur
    rect(v, x0 + 0.5, 0.45, x0 + 3.6, 1.5, blue);
    ellipse(v, vec2(x0 + 1.5, 1.45), 1.1, 0.65, 0.0, white);
    rect(v, x0 + 0.5, 0.45, x0 + 3.6, 0.7, navy);
    rect(v, x0 + 0.85, 2.0, x0 + 1.25, TRAM_TOP, navy); // cheminee
    rect(v, x0 + 0.7, TRAM_TOP - 0.2, x0 + 1.4, TRAM_TOP, navy);
    rect(v, x0 + 2.2, 1.4, x0 + 3.6, TRAM_TOP - 0.15, white); // cabine
    rect(v, x0 + 2.4, 1.7, x0 + 3.4, 2.5, mix(Color::new(0.2, 0.25, 0.3, 1.0), Color::new(1.0, 0.9, 0.6, 1.0), night * 0.8));
    rect(v, x0 + 0.6, TRAM_TOP - 0.15, x0 + 3.8, TRAM_TOP, blue);
    tri(v, vec2(x0, 0.3), vec2(x0 + 0.5, 0.3), vec2(x0 + 0.5, 1.0), navy); // chasse-pigeons
    circ(v, vec2(x0 + 0.45, 1.5), 0.14, Color::new(1.0, 1.0, 0.8, 1.0));
    tyre(v, vec2(x0 + 1.1, 0.3), 0.3, spin);
    tyre(v, vec2(x0 + 3.0, 0.3), 0.3, spin);
    // un peu de fumee (pour faire vrai)
    for k in 0..3 {
        let t = (time * 0.9 + k as f32 * 0.33).fract();
        circ(v, vec2(x0 + 1.05 + t * 1.6, TRAM_TOP + 0.2 + t * 1.2), 0.15 + t * 0.3, Color::new(1.0, 1.0, 1.0, 0.55 * (1.0 - t)));
    }
    // trois wagons ouverts, avec store raye et touristes
    for w in 0..3 {
        let wx = x0 + 4.0 + w as f32 * 4.7;
        line(v, vec2(wx - 0.45, 0.6), vec2(wx, 0.6), 0.1, navy); // attelage
        rect(v, wx, 0.45, wx + 4.3, 1.25, if w % 2 == 0 { white } else { blue });
        rect(v, wx, 0.45, wx + 4.3, 0.65, navy);
        for k in 0..3 {
            let px = wx + 0.1 + k as f32 * 2.0;
            rect(v, px, 1.25, px + 0.1, TRAM_TOP - 0.3, navy);
        }
        // touristes : une tete, un chapeau ou une casquette
        for k in 0..4 {
            let hx = wx + 0.65 + k as f32 * 1.0;
            let skin = [Color::new(0.95, 0.78, 0.65, 1.0), Color::new(0.8, 0.6, 0.45, 1.0), Color::new(0.98, 0.85, 0.75, 1.0)][((hash(w * 4 + k, 91) * 3.0) as usize).min(2)];
            let hat = [Color::new(0.95, 0.3, 0.3, 1.0), Color::new(0.98, 0.9, 0.5, 1.0), Color::new(0.3, 0.7, 0.4, 1.0), Color::new(0.95, 0.55, 0.8, 1.0)][((hash(w * 4 + k, 92) * 4.0) as usize).min(3)];
            let bob = (time * 5.0 + (w * 4 + k) as f32).sin() * 0.03;
            rect(v, hx - 0.22, 1.25, hx + 0.22, 1.6, darker(hat, dk * 0.85));
            circ(v, vec2(hx, 1.8 + bob), 0.2, darker(skin, dk));
            rect(v, hx - 0.28, 1.93 + bob, hx + 0.24, 2.0 + bob, darker(hat, dk));
            half_ellipse(v, hx, 2.0 + bob, 0.18, 0.14, darker(hat, dk));
        }
        // store raye bleu et blanc
        for k in 0..9 {
            let sx = wx - 0.1 + k as f32 * 0.5;
            rect(v, sx, TRAM_TOP - 0.3, sx + 0.5, TRAM_TOP, if k % 2 == 0 { blue } else { white });
        }
        tyre(v, vec2(wx + 0.8, 0.28), 0.28, spin);
        tyre(v, vec2(wx + 3.5, 0.28), 0.28, spin);
    }
}

/// Forcalquier : un tracteur et ses remorques de bottes de foin.
fn draw_hay_tractor(v: &View, x0: f32, night: f32, time: f32) {
    let dk = 1.0 - 0.4 * night;
    let green = darker(Color::new(0.2, 0.55, 0.25, 1.0), dk);
    let green2 = darker(Color::new(0.14, 0.4, 0.18, 1.0), dk);
    let hay = darker(Color::new(0.93, 0.78, 0.35, 1.0), dk);
    let hay2 = darker(Color::new(0.82, 0.66, 0.26, 1.0), dk);
    let spin = time * 4.0;
    // le tracteur : capot, cabine, pot d'echappement
    rect(v, x0 + 0.3, 0.7, x0 + 2.4, 1.7, green);
    rect(v, x0 + 0.3, 0.7, x0 + 2.4, 0.9, green2);
    rect(v, x0 + 0.2, 0.9, x0 + 0.4, 1.5, Color::new(0.15, 0.15, 0.15, 1.0)); // calandre
    rect(v, x0 + 1.0, 1.7, x0 + 1.15, TRAM_TOP - 0.3, Color::new(0.2, 0.2, 0.2, 1.0));
    let puff = (time * 1.4).fract();
    circ(v, vec2(x0 + 1.1 + puff * 1.2, TRAM_TOP - 0.2 + puff), 0.12 + puff * 0.25, Color::new(0.3, 0.3, 0.3, 0.5 * (1.0 - puff)));
    rect(v, x0 + 2.2, 0.9, x0 + 4.0, TRAM_TOP - 0.12, green);
    rect(v, x0 + 2.4, 1.6, x0 + 3.8, 2.55, mix(Color::new(0.55, 0.75, 0.85, 1.0), Color::new(1.0, 0.9, 0.6, 1.0), night * 0.8));
    rect(v, x0 + 2.0, TRAM_TOP - 0.12, x0 + 4.2, TRAM_TOP, green2);
    // le paysan et son chapeau de paille
    circ(v, vec2(x0 + 3.0, 2.05), 0.2, darker(Color::new(0.9, 0.7, 0.55, 1.0), dk));
    rect(v, x0 + 2.68, 2.2, x0 + 3.32, 2.27, hay2);
    half_ellipse(v, x0 + 3.0, 2.27, 0.2, 0.16, hay);
    circ(v, vec2(x0 + 0.35, 1.2), 0.12, Color::new(1.0, 1.0, 0.8, 1.0));
    tyre(v, vec2(x0 + 0.95, 0.42), 0.42, spin);
    tyre(v, vec2(x0 + 3.2, 0.8), 0.8, spin * 0.5);
    // deux remorques chargees de bottes rondes
    for w in 0..2 {
        let wx = x0 + 4.6 + w as f32 * 6.8;
        line(v, vec2(wx - 0.5, 0.6), vec2(wx, 0.6), 0.1, Color::new(0.2, 0.2, 0.2, 1.0));
        rect(v, wx, 0.55, wx + 6.4, 0.85, darker(Color::new(0.55, 0.3, 0.2, 1.0), dk));
        for row in 0..2 {
            for k in 0..6 {
                let bx = wx + 0.55 + k as f32 * 1.06;
                let by = 0.85 + 0.51 + row as f32 * 1.02;
                rect(v, bx - 0.52, by - 0.51, bx + 0.52, by + 0.51, if (k + row) % 2 == 0 { hay } else { hay2 });
                circ(v, vec2(bx, by), 0.45, if (k + row) % 2 == 0 { hay2 } else { hay });
                circ(v, vec2(bx, by), 0.25, if (k + row) % 2 == 0 { hay } else { hay2 });
                circ(v, vec2(bx, by), 0.08, hay2);
            }
        }
        // quelques brins qui depassent
        for k in 0..8 {
            let sx = wx + 0.4 + k as f32 * 0.8;
            line(v, vec2(sx, TRAM_TOP - 0.02), vec2(sx + 0.12, TRAM_TOP + 0.14), 0.03, hay);
        }
        tyre(v, vec2(wx + 1.2, 0.36), 0.36, spin);
        tyre(v, vec2(wx + 5.2, 0.36), 0.36, spin);
    }
}

fn draw_tram(v: &View, t: &Tram, night: f32, time: f32, city: City) {
    match city {
        City::Marseille => return draw_tourist_train(v, t.x, night, time),
        City::Forcalquier => return draw_hay_tractor(v, t.x, night, time),
        _ => {}
    }
    let x0 = t.x;
    let x1 = t.x + TRAM_LEN;
    let dk = 1.0 - 0.4 * night;
    let body = darker(Color::new(0.88, 0.89, 0.9, 1.0), dk);
    let glass = mix(Color::new(0.2, 0.25, 0.3, 1.0), Color::new(1.0, 0.9, 0.6, 1.0), night * 0.8);
    // bogies
    for k in 0..4 {
        let bx = x0 + 1.8 + k as f32 * (TRAM_LEN - 3.6) / 3.0;
        rect(v, bx - 0.8, 0.0, bx + 0.8, 0.35, Color::new(0.2, 0.2, 0.22, 1.0));
        for wx in [bx - 0.45, bx + 0.45] {
            let p = v.p(vec2(wx, 0.2));
            draw_circle(p.x, p.y, v.l(0.2), Color::new(0.1, 0.1, 0.1, 1.0));
        }
    }
    // caisse
    rect(v, x0 + 0.6, 0.3, x1, TRAM_TOP, body);
    // nez arrondi
    let nose = [vec2(x0, 0.35), vec2(x0 + 0.6, 0.3), vec2(x0 + 0.6, TRAM_TOP), vec2(x0 + 0.25, TRAM_TOP - 0.3)];
    tri(v, nose[0], nose[1], nose[2], body);
    tri(v, nose[0], nose[2], nose[3], body);
    // pare-brise
    tri(v, vec2(x0 + 0.08, 1.4), vec2(x0 + 0.9, 1.4), vec2(x0 + 0.9, 2.6), Color::new(0.1, 0.12, 0.15, 1.0));
    tri(v, vec2(x0 + 0.08, 1.4), vec2(x0 + 0.9, 2.6), vec2(x0 + 0.35, 2.55), Color::new(0.1, 0.12, 0.15, 1.0));
    // bandeau bordeaux et fenetres
    rect(v, x0 + 0.3, 0.75, x1, 0.95, darker(BORDEAUX, dk));
    for k in 0..6 {
        let wx = x0 + 1.6 + k as f32 * 2.7;
        if k == 2 || k == 4 {
            // portes
            rect(v, wx, 0.4, wx + 1.3, 2.5, darker(Color::new(0.6, 0.62, 0.65, 1.0), dk));
            rect(v, wx + 0.1, 1.0, wx + 0.6, 2.4, glass);
            rect(v, wx + 0.7, 1.0, wx + 1.2, 2.4, glass);
        } else {
            rect(v, wx, 1.3, wx + 2.2, 2.45, glass);
        }
    }
    // girouette "B"
    rect(v, x0 + 0.95, 2.62, x0 + 1.55, 2.85, Color::new(0.1, 0.1, 0.1, 1.0));
    let sp = v.p(vec2(x0 + 1.25, 2.73));
    let size = (v.s * 0.22) as u16;
    let sign = if city == City::Algerie { "1" } else { "B" };
    comic_text(sign, sp.x, sp.y, size.max(10), Color::new(1.0, 0.5, 0.1, 1.0), INK);
    // phare
    let lp = v.p(vec2(x0 + 0.2, 0.9));
    draw_circle(lp.x, lp.y, v.l(0.1), Color::new(1.0, 1.0, 0.8, 1.0));
    if night > 0.3 || (time * 4.0).sin() > 0.95 {
        let a = v.p(vec2(x0 + 0.1, 0.9));
        let b = v.p(vec2(x0 - 4.0, 0.2));
        let c = v.p(vec2(x0 - 4.0, 1.6));
        draw_triangle(a, b, c, Color::new(1.0, 1.0, 0.8, 0.15));
    }
    // toit et pantographe replie (APS : pas de catenaire !)
    rect(v, x0 + 0.6, TRAM_TOP - 0.08, x1, TRAM_TOP, darker(Color::new(0.7, 0.72, 0.75, 1.0), dk));
    let pb = vec2(x0 + TRAM_LEN * 0.5, TRAM_TOP);
    seg(v, pb, pb + vec2(1.0, 0.3), 0.05, Color::new(0.25, 0.25, 0.28, 1.0));
    seg(v, pb + vec2(1.0, 0.3), pb + vec2(0.2, 0.4), 0.05, Color::new(0.25, 0.25, 0.28, 1.0));
}

/// Un CRS casque, bouclier et matraque, de profil vers la gauche.
/// `ang` : rotation (quand il valdingue), autour du milieu du corps.
fn draw_crs(v: &View, feet: Vec2, ang: f32, phase: f32, standing: bool) {
    let navy = Kind::Crs.color();
    let navy_dark = darker(navy, 0.7);
    let skin = Color::new(0.95, 0.78, 0.65, 1.0);
    let piv = feet + vec2(0.0, 0.6);
    let rot = Vec2::from_angle(ang);
    let q = |lx: f32, ly: f32| piv + rot.rotate(vec2(lx, ly - 0.6));
    let quad = |x0: f32, y0: f32, x1: f32, y1: f32, col: Color| {
        let (a, b, c, d) = (q(x0, y0), q(x1, y0), q(x1, y1), q(x0, y1));
        tri(v, a, b, c, col);
        tri(v, a, c, d, col);
    };
    let walk = if standing { (phase * 7.0).sin() } else { 0.6 };

    // jambes et rangers
    for (dx, o) in [(0.06, 0.0), (-0.06, PI)] {
        let k = if standing { (phase * 7.0 + o).sin() * 0.14 } else { (o.cos()) * 0.2 };
        let hip = q(dx, 0.5);
        let foot = q(dx + k, 0.06);
        seg(v, hip, foot, 0.13, if o == 0.0 { navy_dark } else { navy });
        ellipse(v, foot + rot.rotate(vec2(-0.04, -0.02)), 0.1, 0.06, ang, INK);
    }
    // corps et gilet
    quad(-0.17, 0.45, 0.17, 1.02, navy);
    quad(-0.15, 0.6, 0.15, 0.95, navy_dark);
    quad(-0.17, 0.45, 0.17, 0.52, INK); // ceinturon
    // bras arriere + matraque qui mouline
    let sh = q(0.06, 0.92);
    let swing = if standing { 1.6 + walk * 0.9 } else { 2.6 };
    let dir = rot.rotate(Vec2::from_angle(swing));
    let hand = sh + dir * 0.3;
    seg(v, sh, hand, 0.09, navy);
    seg(v, hand - dir * 0.1, hand + dir * 0.48, 0.06, INK);
    seg(v, hand + rot.rotate(vec2(0.0, 0.0)), hand + rot.rotate(vec2(0.1, -0.03)), 0.05, INK);
    // tete et casque a visiere
    let head = q(-0.03, 1.16);
    circ(v, head, 0.15, skin);
    ellipse(v, q(0.0, 1.21), 0.19, 0.15, ang, navy);
    ellipse(v, q(-0.11, 1.13), 0.085, 0.075, ang, Color::new(0.7, 0.85, 1.0, 0.55));
    seg(v, q(-0.17, 1.2), q(0.17, 1.24), 0.025, Color::new(0.9, 0.9, 0.95, 1.0));
    ellipse(v, q(0.13, 1.08), 0.06, 0.05, ang, navy_dark); // couvre-nuque
    // bouclier devant
    let sc = Color::new(0.75, 0.82, 0.9, 0.55);
    quad(-0.4, 0.28, -0.3, 1.05, sc);
    seg(v, q(-0.4, 0.28), q(-0.4, 1.05), 0.025, Color::new(0.85, 0.9, 0.95, 0.9));
    seg(v, q(-0.12, 0.9), q(-0.32, 0.7), 0.09, navy); // bras du bouclier
    if standing {
        let sp = v.p(q(0.0, 0.78));
        let size = (v.s * 0.15) as u16;
        if size >= 8 {
            let dim = measure_text("CRS", None, size, 1.0);
            draw_text("CRS", sp.x - dim.width / 2.0, sp.y + dim.offset_y / 2.0, size as f32, Color::new(1.0, 0.9, 0.3, 1.0));
        }
    }
}

fn draw_vehicle(v: &View, ve: &Vehicle, night: f32, time: f32) {
    if ve.x < -1.0e5 {
        return;
    }
    let x = ve.x;
    let w = ve.w();
    let dk = 1.0 - 0.35 * night;
    let body = darker(ve.color, dk);
    let glass = mix(Color::new(0.25, 0.32, 0.4, 1.0), Color::new(0.1, 0.1, 0.15, 1.0), night);
    let tire = Color::new(0.08, 0.08, 0.08, 1.0);
    let rim = Color::new(0.7, 0.7, 0.72, 1.0);
    let wheel = |cx: f32, r: f32| {
        let p = v.p(vec2(cx, r));
        draw_circle(p.x, p.y, v.l(r), tire);
        draw_circle(p.x, p.y, v.l(r * 0.5), rim);
        let a = time * 20.0;
        let d = Vec2::from_angle(a) * r * 0.45;
        line(v, vec2(cx, r) - d, vec2(cx, r) + d, 0.03, tire);
    };
    match ve.kind {
        VehKind::Car => {
            // petite citadine, nez a gauche
            rect(v, x + 0.1, 0.25, x + w, 0.95, body);
            ellipse(v, vec2(x + 0.15, 0.6), 0.15, 0.32, 0.0, body);
            let cab = [vec2(x + 0.9, 0.95), vec2(x + w - 0.2, 0.95), vec2(x + w - 0.45, 1.45), vec2(x + 1.45, 1.45)];
            tri(v, cab[0], cab[1], cab[2], body);
            tri(v, cab[0], cab[2], cab[3], body);
            tri(v, vec2(x + 1.05, 1.0), vec2(x + 2.1, 1.0), vec2(x + 2.1, 1.38), glass);
            tri(v, vec2(x + 1.05, 1.0), vec2(x + 2.1, 1.38), vec2(x + 1.5, 1.38), glass);
            rect(v, x + 2.2, 1.0, x + w - 0.45, 1.38, glass);
            line(v, vec2(x + 2.15, 0.3), vec2(x + 2.15, 1.4), 0.03, darker(body, 0.7));
            rect(v, x + 2.4, 0.8, x + 2.65, 0.85, darker(body, 0.6)); // poignee
            rect(v, x, 0.25, x + w + 0.05, 0.38, Color::new(0.2, 0.2, 0.22, 1.0)); // pare-chocs
            let hl = v.p(vec2(x + 0.08, 0.7));
            draw_circle(hl.x, hl.y, v.l(0.08), Color::new(1.0, 1.0, 0.8, 1.0));
            if night > 0.3 {
                tri(v, vec2(x + 0.05, 0.7), vec2(x - 3.0, 0.1), vec2(x - 3.0, 1.2), Color::new(1.0, 1.0, 0.8, 0.12));
            }
            wheel(x + 0.75, 0.32);
            wheel(x + w - 0.75, 0.32);
        }
        VehKind::Moto => {
            wheel(x + 0.35, 0.3);
            wheel(x + w - 0.35, 0.3);
            let frame = Color::new(0.25, 0.25, 0.28, 1.0);
            seg(v, vec2(x + 0.35, 0.3), vec2(x + 0.6, 0.95), 0.06, rim); // fourche
            seg(v, vec2(x + 0.6, 0.55), vec2(x + w - 0.35, 0.35), 0.1, frame);
            ellipse(v, vec2(x + 0.85, 0.75), 0.35, 0.18, -0.1, body); // reservoir
            ellipse(v, vec2(x + 1.4, 0.72), 0.32, 0.1, 0.0, INK); // selle
            ellipse(v, vec2(x + 1.6, 0.5), 0.3, 0.14, 0.0, body); // carenage arriere
            seg(v, vec2(x + 1.1, 0.4), vec2(x + 1.9, 0.45), 0.06, rim); // pot
            // pilote penche en avant
            let rider = Color::new(0.15, 0.15, 0.18, 1.0);
            seg(v, vec2(x + 1.35, 0.8), vec2(x + 1.0, 0.45), 0.1, rider);
            seg(v, vec2(x + 1.4, 0.8), vec2(x + 0.95, 1.15), 0.2, rider);
            seg(v, vec2(x + 1.0, 1.1), vec2(x + 0.6, 0.95), 0.08, rider);
            circ(v, vec2(x + 0.85, 1.2), 0.15, body);
            ellipse(v, vec2(x + 0.76, 1.18), 0.07, 0.06, 0.0, glass);
            let hl = v.p(vec2(x + 0.55, 0.95));
            draw_circle(hl.x, hl.y, v.l(0.06), Color::new(1.0, 1.0, 0.8, 1.0));
        }
    }
}

/// Le char d'assaut, `ox` = arriere. `driver` : None = a l'arret (trappe
/// ouverte), Some(blink) = Acaboulette aux commandes.
fn draw_tank(v: &View, ox: f32, roll: f32, recoil: f32, driver: Option<bool>) {
    let (green, dark) = match driver {
        Some(true) => (Color::new(0.75, 0.8, 0.6, 1.0), Color::new(0.5, 0.55, 0.4, 1.0)),
        _ => (TANK_GREEN, TANK_DARK),
    };
    let steel = Color::new(0.14, 0.14, 0.13, 1.0);
    ellipse(v, vec2(ox + TANK_LEN / 2.0, 0.02), TANK_LEN * 0.6, 0.1, 0.0, Color::new(0.0, 0.0, 0.0, 0.25));

    // Tete du pilote (derriere la tourelle)
    if driver.is_some() {
        let hc = vec2(ox + 2.15, 2.4);
        let hr = 0.32;
        for ex in [-0.2f32, 0.16] {
            tri(v, hc + vec2(ex - 0.11, 0.14), hc + vec2(ex + 0.13, 0.2), hc + vec2(ex + 0.02, 0.5), GINGER);
            tri(v, hc + vec2(ex - 0.05, 0.18), hc + vec2(ex + 0.08, 0.2), hc + vec2(ex + 0.02, 0.4), NOSE);
        }
        ellipse(v, hc, hr * 1.12, hr * 0.95, 0.0, GINGER);
        ellipse(v, hc + vec2(0.18, -0.1), 0.16, 0.12, 0.0, CREAM);
        tri(v, hc + vec2(0.27, 0.0), hc + vec2(0.35, 0.0), hc + vec2(0.31, -0.06), NOSE);
        let eye = hc + vec2(0.11, 0.07);
        ellipse(v, eye, 0.1, 0.11, 0.0, WHITE);
        ellipse(v, eye + vec2(0.03, 0.0), 0.07, 0.085, 0.0, Color::new(0.45, 0.75, 0.2, 1.0));
        ellipse(v, eye + vec2(0.04, 0.0), 0.02, 0.065, 0.0, INK);
        for k in 0..3 {
            let o = hc + vec2(0.24, -0.07 - k as f32 * 0.03);
            line(v, o, o + vec2(0.4, 0.08 - k as f32 * 0.09), 0.012, Color::new(1.0, 1.0, 1.0, 0.9));
        }
        // casque
        half_ellipse(v, hc.x - 0.04, hc.y + 0.12, 0.34, 0.24, dark);
        line(v, hc + vec2(-0.38, 0.12), hc + vec2(0.34, 0.12), 0.05, steel);
    } else {
        // trappe ouverte
        line(v, vec2(ox + 1.85, 2.0), vec2(ox + 1.55, 2.5), 0.09, dark);
    }

    // Le gros joint, au bec
    if driver.is_some() {
        let mouth = vec2(ox + 2.42, 2.27);
        let tip = vec2(ox, 0.0) + TANK_JOINT_TIP;
        let d = (tip - mouth).normalize();
        let n = d.perp();
        let end = tip - d * 0.06;
        // un cone : fin au bec, gros au bout
        tri(v, mouth + n * 0.04, mouth - n * 0.04, end + n * 0.14, Color::new(0.97, 0.96, 0.9, 1.0));
        tri(v, mouth - n * 0.04, end - n * 0.14, end + n * 0.14, Color::new(0.9, 0.88, 0.8, 1.0));
        circ(v, end, 0.14, Color::new(0.35, 0.3, 0.25, 1.0));
        circ(v, tip, 0.1, Color::new(1.0, 0.45 + 0.3 * (roll * 3.0).sin().abs(), 0.1, 1.0));
    }

    // Lance-missiles, a l'arriere de la tourelle
    for k in 0..3 {
        let a = vec2(ox + 0.95 + k as f32 * 0.16, 2.0 + k as f32 * 0.04);
        let b = a + vec2(0.75, 0.48);
        line(v, a, b, 0.15, steel);
        circ(v, b, 0.06, Color::new(0.85, 0.15, 0.1, 1.0));
    }
    rect(v, ox + 1.0, 1.85, ox + 1.6, 2.05, dark);

    // Canon
    let rec = recoil * 0.45;
    let by = 1.72;
    rect(v, ox + 2.9 - rec, by - 0.1, ox + 5.5 - rec, by + 0.1, dark);
    rect(v, ox + 5.3 - rec, by - 0.16, ox + 5.7 - rec, by + 0.16, steel);

    // Tourelle
    ellipse(v, vec2(ox + 2.2, 1.62), 1.15, 0.5, 0.0, green);
    rect(v, ox + 1.25, 1.25, ox + 3.15, 1.62, green);
    ellipse(v, vec2(ox + 3.05, 1.72), 0.2, 0.2, 0.0, dark);
    rect(v, ox + 1.75, 2.0, ox + 2.55, 2.1, dark); // trappe

    // Caisse
    rect(v, ox + 0.35, 0.6, ox + 4.2, 1.3, green);
    tri(v, vec2(ox + 4.2, 0.6), vec2(ox + 4.2, 1.3), vec2(ox + 4.7, 0.75), green);
    tri(v, vec2(ox + 0.35, 0.6), vec2(ox + 0.35, 1.3), vec2(ox + 0.05, 0.8), green);
    rect(v, ox + 0.35, 0.6, ox + 4.2, 0.75, dark);
    // etoile de la 2e DB (on est pas loin de la place de la Victoire)
    let sp = v.p(vec2(ox + 1.2, 1.02));
    draw_poly(sp.x, sp.y, 5, v.l(0.17), -90.0, Color::new(0.95, 0.93, 0.84, 0.9));

    // Chenilles
    rect(v, ox + 0.4, 0.0, ox + 4.2, 0.72, steel);
    circ(v, vec2(ox + 0.4, 0.36), 0.36, steel);
    circ(v, vec2(ox + 4.2, 0.36), 0.36, steel);
    for k in 0..6 {
        let c = vec2(ox + 0.45 + k as f32 * 0.74, 0.34);
        circ(v, c, 0.27, Color::new(0.32, 0.34, 0.3, 1.0));
        circ(v, c, 0.1, steel);
        let d = Vec2::from_angle(-roll + k as f32) * 0.24;
        line(v, c - d, c + d, 0.04, steel);
    }
    // maillons qui defilent
    for k in 0..12 {
        let x = ox + 0.4 + ((k as f32 * 0.33 - roll * 0.25).rem_euclid(3.8));
        line(v, vec2(x, 0.0), vec2(x, 0.07), 0.05, Color::new(0.3, 0.3, 0.28, 1.0));
        line(v, vec2(x, 0.65), vec2(x, 0.72), 0.05, Color::new(0.3, 0.3, 0.28, 1.0));
    }
}

/// Fleche verte au-dessus du char a l'arret : il faut sauter dedans.
fn draw_tank_arrow(v: &View, ox: f32, time: f32) {
    let bob = (time * 5.0).sin() * 0.22;
    let c = vec2(ox + 2.15, 3.35 + bob);
    let edge = Color::new(0.03, 0.25, 0.05, 1.0);
    let fill = Color::new(0.25, 0.9, 0.25, 1.0);
    for (k, col) in [(1.0f32, edge), (0.78, fill)] {
        let hw = 0.5 * k;
        let lift = (1.0 - k) * 0.35;
        rect(v, c.x - 0.2 * k, c.y + 0.4, c.x + 0.2 * k, c.y + 1.15 - lift * 0.4, col);
        tri(v, vec2(c.x - hw, c.y + 0.45 - lift * 0.2), vec2(c.x + hw, c.y + 0.45 - lift * 0.2), vec2(c.x, c.y - 0.3 + lift), col);
    }
    let sp = v.p(vec2(c.x, c.y + 1.55));
    comic_text("SAUTE DEDANS !", sp.x, sp.y, (v.l(0.42) as u16).max(12), fill, edge);
}

fn draw_shell(v: &View, p: Vec2, time: f32) {
    let fl = 0.9 + (time * 50.0).sin().abs() * 0.7;
    tri(v, p + vec2(-0.5, 0.2), p + vec2(-0.5, -0.2), p + vec2(-0.5 - fl, 0.0), Color::new(1.0, 0.5, 0.05, 0.85));
    tri(v, p + vec2(-0.5, 0.12), p + vec2(-0.5, -0.12), p + vec2(-0.5 - fl * 0.6, 0.0), Color::new(1.0, 0.9, 0.3, 0.9));
    rect(v, p.x - 0.55, p.y - 0.24, p.x + 0.3, p.y + 0.24, Color::new(0.25, 0.26, 0.28, 1.0));
    ellipse(v, p + vec2(0.3, 0.0), 0.5, 0.24, 0.0, Color::new(0.25, 0.26, 0.28, 1.0));
    rect(v, p.x - 0.3, p.y - 0.24, p.x - 0.15, p.y + 0.24, Color::new(0.85, 0.65, 0.15, 1.0));
    line(v, p + vec2(-0.5, 0.14), p + vec2(0.45, 0.14), 0.04, Color::new(0.5, 0.52, 0.55, 1.0));
}

fn draw_missile(v: &View, m: &Missile, time: f32) {
    let d = m.vel.normalize_or_zero();
    let n = d.perp();
    let (tail, tip) = (m.pos - d * 0.32, m.pos + d * 0.3);
    let fl = 0.25 + (time * 60.0 + m.age * 9.0).sin().abs() * 0.3;
    tri(v, tail + n * 0.08, tail - n * 0.08, tail - d * fl, Color::new(1.0, 0.6, 0.1, 0.9));
    tri(v, tail + n * 0.2, tail - n * 0.2, tail + d * 0.2, Color::new(0.7, 0.1, 0.1, 1.0)); // ailettes
    seg(v, tail, tip, 0.15, Color::new(0.93, 0.93, 0.95, 1.0));
    tri(v, tip + n * 0.075, tip - n * 0.075, tip + d * 0.2, Color::new(0.85, 0.15, 0.1, 1.0));
}

fn draw_critter(v: &View, c: &Critter, time: f32) {
    let p = c.pos;
    let ph = c.phase;
    match c.kind {
        Kind::Pigeon => {
            let grey = Kind::Pigeon.color();
            let dark = Color::new(0.38, 0.4, 0.46, 1.0);
            let bob = if c.flying { 0.0 } else { (ph * 7.0).sin() * 0.04 };
            let peck = !c.flying && (ph * 0.9).sin() > 0.85;
            let b = p + vec2(0.0, 0.22);
            if !c.flying {
                for dx in [-0.05, 0.06] {
                    let k = (ph * 7.0 + dx * 30.0).sin() * 0.05;
                    line(v, b + vec2(dx, -0.08), p + vec2(dx + k, 0.0), 0.03, Color::new(0.9, 0.45, 0.45, 1.0));
                }
            }
            tri(v, b + vec2(0.15, 0.04), b + vec2(0.38, 0.08), b + vec2(0.36, -0.04), dark); // queue
            ellipse(v, b, 0.24, 0.15, 0.1, grey);
            // ailes
            if c.flying {
                let flap = (time * 25.0 + ph).sin();
                tri(v, b + vec2(-0.05, 0.05), b + vec2(0.15, 0.05), b + vec2(0.1, 0.05 + flap * 0.4), dark);
            } else {
                ellipse(v, b + vec2(0.05, 0.03), 0.15, 0.08, 0.15, dark);
                line(v, b + vec2(-0.02, 0.0), b + vec2(0.15, 0.02), 0.02, Color::new(0.2, 0.2, 0.25, 1.0));
            }
            let head = if peck { b + vec2(-0.25, -0.02) } else { b + vec2(-0.2, 0.15 + bob) };
            seg(v, b + vec2(-0.12, 0.05), head, 0.12, Color::new(0.35, 0.55, 0.5, 1.0)); // cou irise
            circ(v, head, 0.08, grey);
            tri(v, head + vec2(-0.06, 0.02), head + vec2(-0.06, -0.02), head + vec2(-0.14, -0.01), Color::new(0.3, 0.3, 0.3, 1.0));
            circ(v, head + vec2(-0.02, 0.02), 0.022, Color::new(0.95, 0.5, 0.1, 1.0));
        }
        Kind::Crs => draw_crs(v, p, 0.0, ph, true),
        Kind::Gull => {
            let white = Kind::Gull.color();
            let grey = Color::new(0.62, 0.66, 0.72, 1.0);
            let black = Color::new(0.12, 0.12, 0.15, 1.0);
            let b = p + vec2(0.0, 0.3);
            let flap = (time * 9.0 + ph).sin();
            // aile du fond
            let far = b + vec2(0.12, 0.08 + flap * 0.5);
            tri(v, b + vec2(-0.12, 0.05), b + vec2(0.2, 0.05), far, darker(grey, 0.85));
            tri(v, b + vec2(0.3, 0.02), b + vec2(0.58, 0.08), b + vec2(0.55, -0.05), white); // queue
            ellipse(v, b, 0.36, 0.15, 0.06, white);
            let head = b + vec2(-0.34, 0.1);
            circ(v, head, 0.12, white);
            tri(v, head + vec2(-0.09, 0.04), head + vec2(-0.09, -0.05), head + vec2(-0.3, -0.03), Color::new(1.0, 0.75, 0.1, 1.0));
            circ(v, head + vec2(-0.03, 0.03), 0.025, black);
            // aile de devant, au bout noir
            let tip = b + vec2(0.2, 0.06 + flap * 0.62);
            let mid = b.lerp(tip, 0.62);
            tri(v, b + vec2(-0.16, 0.04), b + vec2(0.22, 0.04), tip, grey);
            tri(v, mid + vec2(-0.06, 0.0), mid + vec2(0.07, 0.0), tip, black);
        }
        Kind::Rat => {
            let fur = Kind::Rat.color();
            let run = (ph * 18.0).sin();
            let b = p + vec2(0.0, 0.17);
            // queue rose
            let mut prev = b + vec2(0.25, -0.02);
            for k in 1..=5 {
                let q = prev + vec2(0.1, (ph * 10.0 + k as f32).sin() * 0.03 - 0.01);
                line(v, prev, q, 0.03, Color::new(0.9, 0.6, 0.6, 1.0));
                prev = q;
            }
            for dx in [-0.12, 0.12] {
                line(v, b + vec2(dx, -0.05), p + vec2(dx + run * 0.05 * dx.signum(), 0.0), 0.04, darker(fur, 0.7));
            }
            ellipse(v, b, 0.27, 0.14, 0.0, fur);
            ellipse(v, b + vec2(-0.25, 0.02), 0.13, 0.09, -0.2, fur);
            circ(v, b + vec2(-0.2, 0.12), 0.06, Color::new(0.85, 0.6, 0.6, 1.0));
            circ(v, b + vec2(-0.37, 0.0), 0.025, Color::new(0.9, 0.5, 0.55, 1.0));
            circ(v, b + vec2(-0.27, 0.05), 0.022, Color::new(0.9, 0.1, 0.1, 1.0));
            // dents
            rect(v, b.x - 0.35, b.y - 0.07, b.x - 0.33, b.y - 0.02, WHITE);
        }
        Kind::Dog => {
            // Chien : quelques robes au hasard (base_y sert de graine)
            let robes = [
                Color::new(0.72, 0.5, 0.3, 1.0),
                Color::new(0.25, 0.2, 0.18, 1.0),
                Color::new(0.95, 0.92, 0.85, 1.0),
                Color::new(0.55, 0.35, 0.2, 1.0),
            ];
            let fur = robes[((c.base_y * 4.0) as usize).min(3)];
            let dark = darker(fur, 0.7);
            let run = ph * 14.0;
            let b = p + vec2(0.0, 0.38);
            // queue qui fretille
            let wag = (time * 18.0).sin() * 0.25;
            seg(v, b + vec2(0.3, 0.05), b + vec2(0.48, 0.3 + wag * 0.3), 0.06, fur);
            for (dx, o) in [(-0.2, 0.0), (0.2, PI), (-0.12, PI), (0.26, 0.0)] {
                let k = (run + o).sin() * 0.1;
                seg(v, b + vec2(dx, -0.05), p + vec2(dx + k, 0.03), 0.08, if o == 0.0 { dark } else { fur });
            }
            ellipse(v, b, 0.34, 0.17, 0.0, fur);
            // collier bordeaux
            seg(v, b + vec2(-0.3, 0.02), b + vec2(-0.26, 0.2), 0.06, BORDEAUX);
            let h = b + vec2(-0.38, 0.2 + (run * 0.5).sin().abs() * 0.03);
            circ(v, h, 0.14, fur);
            ellipse(v, h + vec2(-0.14, -0.04), 0.1, 0.07, 0.0, fur); // museau
            circ(v, h + vec2(-0.24, -0.02), 0.035, INK);
            circ(v, h + vec2(-0.04, 0.04), 0.025, INK);
            ellipse(v, h + vec2(0.06, 0.02), 0.06, 0.12, 0.3, dark); // oreille
            // langue
            ellipse(v, h + vec2(-0.16, -0.12), 0.03, 0.05, 0.0, NOSE);
        }
    }
}

// ---------------------------------------------------------------------------

fn conf() -> Conf {
    Conf {
        window_title: "Acaboulette".to_owned(),
        window_width: 1200,
        window_height: 700,
        high_dpi: true,
        sample_count: 4,
        icon: window_icon(),
        ..Default::default()
    }
}

/// Icone de la fenetre (barre de titre et barre des taches sous Windows).
fn window_icon() -> Option<miniquad::conf::Icon> {
    fn rgba<const N: usize>(png: &[u8]) -> Option<[u8; N]> {
        let img = Image::from_file_with_format(png, Some(ImageFormat::Png)).ok()?;
        img.bytes.try_into().ok()
    }
    Some(miniquad::conf::Icon {
        small: rgba(include_bytes!("../data/icons/packarch-acaboulette/16.png"))?,
        medium: rgba(include_bytes!("../data/icons/packarch-acaboulette/32.png"))?,
        big: rgba(include_bytes!("../data/icons/packarch-acaboulette/64.png"))?,
    })
}

/// Bouton de l'ecran de licence ; renvoie vrai s'il vient d'etre clique.
fn licence_button(label: &str, x: f32, y: f32, w: f32, h: f32, base: Color) -> bool {
    let (mx, my) = mouse_position();
    let hover = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let k = if hover { 1.25 } else { 1.0 };
    let col = Color::new((base.r * k).min(1.0), (base.g * k).min(1.0), (base.b * k).min(1.0), 1.0);
    round_rect(x - 3.0, y - 3.0, w + 6.0, h + 6.0, 15.0, Color::new(0.1, 0.0, 0.03, 1.0));
    round_rect(x, y, w, h, 12.0, col);
    comic_text(label, x + w / 2.0, y + h / 2.0, 32, WHITE, Color::new(0.1, 0.0, 0.03, 1.0));
    hover && is_mouse_button_pressed(MouseButton::Left)
}

const PROJECT_URL: &str = "https://github.com/packarch/packarch-acaboulette";

/// Ouvre une adresse dans le navigateur par defaut.
fn open_url(url: &str) {
    #[cfg(windows)]
    let cmd = std::process::Command::new("explorer").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let cmd = std::process::Command::new("open").arg(url).spawn();
    #[cfg(not(any(windows, target_os = "macos")))]
    let cmd = std::process::Command::new("xdg-open").arg(url).spawn();
    let _ = cmd;
}

/// Lien souligne centre sur l'ecran ; renvoie vrai s'il vient d'etre clique.
fn licence_link(txt: &str, y: f32, size: u16) -> bool {
    let dim = measure_text(txt, None, size, 1.0);
    let x = screen_width() / 2.0 - dim.width / 2.0;
    let (mx, my) = mouse_position();
    let hover = mx >= x - 6.0 && mx <= x + dim.width + 6.0 && my >= y - dim.offset_y - 6.0 && my <= y + 10.0;
    let col = if hover { WHITE } else { Color::new(0.55, 0.8, 1.0, 1.0) };
    draw_text(txt, x, y, size as f32, col);
    draw_line(x, y + 5.0, x + dim.width, y + 5.0, 2.0, col);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// Fenetre de demarrage : la licence, a accepter (vrai) ou refuser (faux).
async fn licence_screen() -> bool {
    let lines = [
        "Copyright (C) 2026 Acaboulette Legrand",
        "",
        "Ce jeu est un logiciel libre, distribué sous la licence",
        "publique générale GNU, version 3 (GPL-3.0) ou ultérieure.",
        "Tu peux le copier, l'étudier, le modifier et le redistribuer",
        "selon les termes de cette licence.",
        "",
        "Il est fourni SANS AUCUNE GARANTIE, pas même pour les",
        "pigeons écrabouillés. Le texte complet de la licence se",
        "trouve dans le fichier COPYING.",
    ];
    let mut time = 0.0f32;
    loop {
        time += get_frame_time().min(0.1);
        let w = screen_width();
        let h = screen_height();
        comic_rays(Color::new(0.33, 0.05, 0.13, 1.0), Color::new(0.4, 0.07, 0.16, 1.0), time);

        let pw = 720.0f32.min(w - 32.0);
        let ph = 570.0f32.min(h - 32.0);
        let px = w / 2.0 - pw / 2.0;
        let py = h / 2.0 - ph / 2.0;
        round_rect(px, py, pw, ph, 16.0, Color::new(0.2, 0.03, 0.07, 0.92));
        comic_text("ACABOULETTE", w / 2.0, py + 58.0, 64, GINGER, Color::new(0.3, 0.05, 0.02, 1.0));
        centered("Licence", py + 116.0, 28, Color::new(1.0, 0.9, 0.7, 1.0));
        for (i, l) in lines.iter().enumerate() {
            let col = if i == 0 { Color::new(1.0, 0.85, 0.2, 1.0) } else { Color::new(1.0, 1.0, 1.0, 0.9) };
            centered(l, py + 160.0 + i as f32 * 26.0, 22, col);
        }

        centered("Le code source est sur GitHub :", py + 436.0, 22, Color::new(1.0, 1.0, 1.0, 0.9));
        if licence_link(PROJECT_URL, py + 464.0, 22) {
            open_url(PROJECT_URL);
        }

        let bw = 220.0f32.min((pw - 72.0) / 2.0);
        let bh = 60.0;
        let by = py + ph - bh - 28.0;
        let accept = licence_button("Accepter", w / 2.0 - bw - 16.0, by, bw, bh, Color::new(0.16, 0.55, 0.2, 1.0));
        let refuse = licence_button("Refuser", w / 2.0 + 16.0, by, bw, bh, Color::new(0.65, 0.12, 0.12, 1.0));
        let accept = accept || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        let refuse = refuse || is_key_pressed(KeyCode::Escape);

        // On attend l'image suivante : le clic ne doit pas aussi lancer la partie.
        next_frame().await;
        if refuse {
            return false;
        }
        if accept {
            return true;
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    if !licence_screen().await {
        return;
    }
    macroquad::rand::srand(macroquad::miniquad::date::now() as u64);
    let mut game = Game::new(load_record(), true);
    let mut acc = 0.0;
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        let frame = get_frame_time().min(0.1);
        game.input();
        acc += frame;
        let mut steps = 0;
        while acc >= DT && steps < 20 {
            game.step(DT);
            acc -= DT;
            steps += 1;
        }

        clear_background(BLACK);
        game.draw();
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_icon_decodes() {
        assert!(window_icon().is_some());
    }

    #[test]
    fn les_quartiers_tournent_en_boucle() {
        assert!(zone_at(0.0) == Zone::Chartrons);
        assert!(zone_at(ZONE_LEN * 2.5) == Zone::PontDePierre);
        // apres Bordeaux, Marseille ; puis retour a Bordeaux
        assert!(zone_at(ZONE_LEN * 7.1) == Zone::VieuxPort);
        assert!(zone_at(ZONE_LEN * 12.5) == Zone::Corniche);
        assert!(zone_at(ZONE_LEN * 13.1) == Zone::Bourguet);
        assert!(zone_at(ZONE_LEN * 16.1) == Zone::Oliviers);
        assert!(zone_at(ZONE_LEN * 18.1) == Zone::GrandePoste);
        assert!(zone_at(ZONE_LEN * 23.1) == Zone::Chartrons);
        assert!(zone_at(-1.0) == Zone::Sahara);
        // chaque ville tient d'un seul bloc, dans l'ordre du voyage
        let order: Vec<City> = ZONES.iter().map(|z| z.city()).collect();
        let mut blocks = order.clone();
        blocks.dedup();
        assert_eq!(blocks, CITIES.to_vec());
        assert_eq!(City::Marseille.first_zone(), 7);
        assert_eq!(City::Forcalquier.first_zone(), 13);
        assert_eq!(City::Algerie.first_zone(), 18);
    }

    #[test]
    fn le_bruit_reste_dans_l_intervalle() {
        for i in -500..500 {
            let h = hash(i, 7);
            assert!((0.0..1.0).contains(&h));
        }
    }

    #[test]
    fn la_gravite_finit_toujours_par_gagner() {
        // Deltaplane : meme en martelant le clic, on ne gagne que quelques
        // metres et l'aile finit par se replier.
        for period in [1usize, 6, 24, 60, 120] {
            let (mut y, mut vy, mut t, mut top) = (0.0f32, 2.0f32, 0.0f32, 0.0f32);
            let mut steps = 0;
            while t < GLIDE_MAX {
                if steps % period == 0 {
                    vy = vy.max(glide_lift(t));
                    t += GLIDE_FLARE_COST;
                }
                t += DT;
                vy = (vy - GLIDE_GRAV * DT).max(-glide_sink(t));
                y += vy * DT;
                top = top.max(y);
                steps += 1;
            }
            assert!(steps as f32 * DT <= GLIDE_MAX, "le vol dure trop longtemps");
            assert!(top < 5.0, "clic toutes les {period} images : +{top} m");
        }
        // Sans rien toucher, on descend tout du long.
        assert!(glide_sink(0.0) > 0.0 && glide_sink(GLIDE_MAX) > glide_sink(0.0));
        assert_eq!(glide_lift(GLIDE_MAX), 0.0);
    }

    #[test]
    fn le_char_pulverise_le_tram() {
        let mut g = Game::new(0, false);
        g.phase = Phase::Running;
        g.critters.clear();
        g.pos = vec2(1000.0, 0.0);
        g.next_spawn = 1.0e9; // rien d'autre sur la route
        g.tank_spots.push(TankSpot { x: 1004.0 });
        // on court dans le char sans sauter : il ne se passe rien
        for _ in 0..120 {
            g.step(DT);
        }
        assert_eq!(g.tank, 0.0);
        // en sautant dedans, on le prend
        g.tank_spots.push(TankSpot { x: g.pos.x + 3.0 });
        g.vy = JUMP_V;
        g.on_ground = false;
        for _ in 0..120 {
            g.step(DT);
        }
        assert!(g.tank > 0.0 && g.on_ground && g.pos.y == 0.0);
        // un obus pulverise le tram d'en face, un autre tram est pulverise au contact
        g.trams.push(Tram { x: g.pos.x + 12.0, warned: true, rode: false });
        g.fire_shell();
        for _ in 0..120 {
            g.step(DT);
        }
        assert!(g.trams.is_empty() && g.shells.is_empty());
        g.trams.push(Tram { x: g.pos.x + 6.0, warned: true, rode: false });
        for _ in 0..240 {
            g.step(DT);
        }
        assert!(g.trams.is_empty() && g.phase == Phase::Running && g.lives == LIVES);
        // les missiles vont chercher les bestioles, meme les mouettes
        g.spawn_critter(Kind::Pigeon, g.pos.x + 9.0);
        g.spawn_critter(Kind::Gull, g.pos.x + 12.0);
        g.fire_missiles();
        assert_eq!(g.missiles.len(), MISSILE_N);
        for _ in 0..300 {
            g.step(DT);
        }
        assert!(g.critters.is_empty() && g.missiles.is_empty());
        assert_eq!(g.kills[Kind::Gull as usize], 1);
        // le plein ne dure pas : Acaboulette finit par en ressortir, entier
        for _ in 0..(TANK_TIME / DT) as usize {
            g.step(DT);
        }
        assert!(g.tank == 0.0 && g.phase == Phase::Running);
    }

    #[test]
    fn le_saut_tient_sur_le_toit_du_tram() {
        // Saut maintenu : on doit pouvoir monter sur le toit du tram B.
        let h_hold = JUMP_V * JUMP_V / (2.0 * GRAV * HOLD_FACTOR);
        assert!(h_hold > TRAM_TOP + 0.5);
        // Saut simple + double saut aussi.
        let h1 = JUMP_V * JUMP_V / (2.0 * GRAV);
        let v2 = JUMP_V * 0.85;
        assert!(h1 + v2 * v2 / (2.0 * GRAV) > TRAM_TOP);
    }
}
