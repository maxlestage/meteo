//! Le moteur du film : des milliers de grains dessinés par WebGL 2.
//!
//! Tout le travail est dans les shaders. Le processeur pose une fois pour
//! toutes la place de chaque grain à chaque station (`formes::film`) ; à
//! chaque image, il ne dit plus que de quelle station à quelle station l'on
//! va, et où l'on en est. Le passage éclate les grains puis les reforme ; la
//! pluie tombe, le radar balaie et les sources votent sans que le processeur
//! ne recalcule rien.

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlBuffer, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};

use super::formes::{Graine, STATIONS, film};

const SOMMETS: &str = r#"#version 300 es
layout(location = 0) in vec4 a_de;
layout(location = 1) in vec4 a_vers;
layout(location = 2) in vec4 a_hasard;
uniform float u_t;
uniform float u_temps;
uniform vec2 u_echelle;
uniform vec2 u_centre;
uniform float u_taille;
uniform float u_mouvement;
out vec3 v_couleur;
out float v_alpha;

// Crème, pluie, soleil, feuille : les couleurs de la marque.
const vec3 TEINTES[4] = vec3[4](
  vec3(0.95, 0.94, 0.89),
  vec3(0.50, 0.82, 0.97),
  vec3(0.98, 0.79, 0.30),
  vec3(0.73, 0.85, 0.56)
);

// La place d'un grain à sa station, avec son mouvement propre ; z de la
// sortie : ce que le mouvement retire d'éclat (une goutte qui finit sa chute).
vec3 place(vec4 g, vec4 h) {
  vec2 p = g.xy;
  float t = u_temps * u_mouvement;
  float eclat = 1.0;
  if (g.w > 0.5 && g.w < 1.5) {
    // La pluie : chaque grain glisse le long de son filet, puis repart.
    float d = fract(h.x + t * (0.45 + 0.35 * h.y));
    p += vec2(-0.03, -0.24) * d;
    eclat = sin(3.14159 * d);
  } else if (g.w > 1.5 && g.w < 2.5) {
    // Le radar : le balayage tourne, et laisse une traînée.
    float a = t * 1.2 + h.x * 0.35;
    p = mat2(cos(a), sin(a), -sin(a), cos(a)) * p;
    eclat = 1.0 - h.x * 0.8;
  } else if (g.w > 2.5) {
    // Les sources : chaque grain file vers le centre, où l'on vote.
    float r = length(p);
    float f = fract((0.74 - r) / 0.5 + t * 0.35);
    p = normalize(p) * mix(0.74, 0.24, f);
    eclat = sin(3.14159 * f);
  }
  // Rien ne tient tout à fait en place : un frémissement.
  p += 0.005 * u_mouvement * vec2(sin(u_temps * (0.6 + h.x) + h.y * 40.0), cos(u_temps * (0.5 + h.y) + h.x * 40.0));
  return vec3(p, eclat);
}

void main() {
  // Chaque grain part avec son retard : la forme se défait par morceaux.
  float local = clamp((u_t - a_hasard.w * 0.35) / 0.65, 0.0, 1.0);
  float e = local * local * (3.0 - 2.0 * local);
  vec3 a = place(a_de, a_hasard);
  vec3 b = place(a_vers, a_hasard);
  vec2 eclate = (a_hasard.xy * 2.0 - 1.0) * vec2(0.7, 0.45) * sin(3.14159 * local);
  vec2 p = mix(a.xy, b.xy, e) + eclate;
  gl_Position = vec4(p * u_echelle + u_centre, 0.0, 1.0);

  bool paleA = fract(a_de.z) > 0.25;
  bool paleB = fract(a_vers.z) > 0.25;
  v_couleur = mix(TEINTES[int(a_de.z)], TEINTES[int(a_vers.z)], e);
  float alphaA = (paleA ? 0.4 : 0.95) * a.z;
  float alphaB = (paleB ? 0.4 : 0.95) * b.z;
  v_alpha = mix(alphaA, alphaB, e);
  float tailleA = paleA ? 0.85 : 1.0;
  float tailleB = paleB ? 0.85 : 1.0;
  gl_PointSize = u_taille * mix(tailleA, tailleB, e) * (0.7 + 0.6 * a_hasard.z);
}
"#;

const FRAGMENTS: &str = r#"#version 300 es
precision mediump float;
in vec3 v_couleur;
in float v_alpha;
out vec4 couleur;
void main() {
  float r = length(gl_PointCoord - 0.5);
  float a = smoothstep(0.5, 0.05, r) * v_alpha;
  couleur = vec4(v_couleur * a, a);
}
"#;

pub struct Moteur {
    gl: Gl,
    vao: WebGlVertexArrayObject,
    tampon: WebGlBuffer,
    n: i32,
    u_t: Option<WebGlUniformLocation>,
    u_temps: Option<WebGlUniformLocation>,
    u_echelle: Option<WebGlUniformLocation>,
    u_centre: Option<WebGlUniformLocation>,
    u_taille: Option<WebGlUniformLocation>,
    u_mouvement: Option<WebGlUniformLocation>,
}

fn shader(gl: &Gl, genre: u32, source: &str) -> Option<web_sys::WebGlShader> {
    let s = gl.create_shader(genre)?;
    gl.shader_source(&s, source);
    gl.compile_shader(&s);
    if gl.get_shader_parameter(&s, Gl::COMPILE_STATUS).as_bool() == Some(true) {
        Some(s)
    } else {
        web_sys::console::warn_1(&gl.get_shader_info_log(&s).unwrap_or_default().into());
        None
    }
}

fn programme(gl: &Gl) -> Option<WebGlProgram> {
    let p = gl.create_program()?;
    gl.attach_shader(&p, &shader(gl, Gl::VERTEX_SHADER, SOMMETS)?);
    gl.attach_shader(&p, &shader(gl, Gl::FRAGMENT_SHADER, FRAGMENTS)?);
    gl.link_program(&p);
    (gl.get_program_parameter(&p, Gl::LINK_STATUS).as_bool() == Some(true)).then_some(p)
}

fn charger(gl: &Gl, donnees: &[f32]) -> Option<WebGlBuffer> {
    let tampon = gl.create_buffer()?;
    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&tampon));
    let vue = js_sys::Float32Array::from(donnees);
    gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &vue, Gl::STATIC_DRAW);
    Some(tampon)
}

impl Moteur {
    /// Le moteur, ou rien si le navigateur ne sait pas faire de WebGL 2 :
    /// la page montre alors les phrases du film sans les grains.
    pub fn nouveau(canvas: &HtmlCanvasElement, n: usize) -> Option<Moteur> {
        let options = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&options, &"antialias".into(), &false.into());
        let _ = js_sys::Reflect::set(&options, &"premultipliedAlpha".into(), &true.into());
        let gl: Gl = canvas.get_context_with_context_options("webgl2", &options).ok()??.dyn_into().ok()?;
        let p = programme(&gl)?;
        gl.use_program(Some(&p));

        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(&vao));

        // Le hasard de chaque grain : son retard, son éclat, sa taille.
        let mut graine = Graine(48_856);
        let hasard: Vec<f32> = (0..n * 4).map(|_| graine.suivant()).collect();
        charger(&gl, &hasard)?;
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_with_i32(2, 4, Gl::FLOAT, false, 16, 0);

        let tampon = charger(&gl, &film(n))?;
        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);

        gl.enable(Gl::BLEND);
        gl.blend_func(Gl::ONE, Gl::ONE);

        let u = |nom: &str| gl.get_uniform_location(&p, nom);
        Some(Moteur {
            u_t: u("u_t"),
            u_temps: u("u_temps"),
            u_echelle: u("u_echelle"),
            u_centre: u("u_centre"),
            u_taille: u("u_taille"),
            u_mouvement: u("u_mouvement"),
            gl,
            vao,
            tampon,
            n: n as i32,
        })
    }

    /// Une image : de la station `de` à la station `vers`, avancée `t`.
    /// `largeur` et `hauteur` en pixels de l'écran, `dpr` la densité.
    #[allow(clippy::too_many_arguments)]
    pub fn dessiner(&self, de: usize, vers: usize, t: f32, temps: f32, largeur: f32, hauteur: f32, dpr: f32, mouvement: bool) {
        let gl = &self.gl;
        gl.viewport(0, 0, (largeur * dpr) as i32, (hauteur * dpr) as i32);
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(Gl::COLOR_BUFFER_BIT);

        gl.bind_vertex_array(Some(&self.vao));
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.tampon));
        let pas = self.n * 16;
        let de = de.min(STATIONS.len() - 1) as i32;
        let vers = vers.min(STATIONS.len() - 1) as i32;
        gl.vertex_attrib_pointer_with_i32(0, 4, Gl::FLOAT, false, 16, de * pas);
        gl.vertex_attrib_pointer_with_i32(1, 4, Gl::FLOAT, false, 16, vers * pas);

        let (echelle, centre) = cadrage(largeur, hauteur);
        gl.uniform1f(self.u_t.as_ref(), t);
        gl.uniform1f(self.u_temps.as_ref(), temps);
        gl.uniform2f(self.u_echelle.as_ref(), echelle.0, echelle.1);
        gl.uniform2f(self.u_centre.as_ref(), centre.0, centre.1);
        gl.uniform1f(self.u_taille.as_ref(), (2.2 * dpr).max(1.5));
        gl.uniform1f(self.u_mouvement.as_ref(), if mouvement { 1.0 } else { 0.0 });
        gl.draw_arrays(Gl::POINTS, 0, self.n);
    }
}

/// Du repère des formes (x ±1,6, y ±1) à l'écran : la forme tient dans la
/// largeur et dans les deux tiers hauts, le bas étant laissé au texte. Sur un
/// écran en hauteur, la largeur est courte : la forme descend vers sa phrase
/// plutôt que de flotter loin d'elle.
pub fn cadrage(largeur: f32, hauteur: f32) -> ((f32, f32), (f32, f32)) {
    let unite = (largeur * 0.96 / 3.2).min(hauteur * 0.56 / 2.0);
    let centre = if hauteur > largeur * 1.3 { 0.04 } else { 0.16 };
    ((unite * 2.0 / largeur, unite * 2.0 / hauteur), (0.0, centre))
}

#[cfg(test)]
mod tests {
    use super::cadrage;

    #[test]
    fn la_forme_tient_dans_l_ecran_et_laisse_le_bas_au_texte() {
        for (l, h) in [(1280.0, 800.0), (390.0, 844.0), (768.0, 1024.0), (1920.0, 1080.0)] {
            let ((ex, ey), (_, cy)) = cadrage(l, h);
            // Les bords du cadre des formes, en coordonnées d'écran (-1 à 1).
            assert!(1.6 * ex <= 1.0, "{l}×{h} : trop large");
            assert!(cy + ey <= 1.0, "{l}×{h} : sort par le haut");
            // Le bas de la forme reste au-dessus du dernier quart de l'écran.
            assert!(cy - ey >= -0.5, "{l}×{h} : descend sur le texte");
        }
    }
}
