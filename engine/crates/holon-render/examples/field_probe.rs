//! FIELD-1 diagnostic: the S1 scene with the field ON, sampled every 500 steps.
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::water_centers;
use holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::{Boundary, Dims, Sim};
#[path = "../tests/common/quartet.rs"]
mod quartet;
fn four_waters() -> Box<Sim> {
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let oxygens = [[5.0, 5.0, 5.0], [12.0, 5.0, 5.0], [5.0, 12.0, 5.0], [12.0, 12.0, 5.0]];
    let (mut species, mut pos) = (Vec::new(), Vec::new());
    for (k, o) in oxygens.iter().enumerate() {
        let flip = if k % 2 == 0 { 1.0 } else { -1.0 };
        for (m, c) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([o[0] + flip * c[0], o[1] + c[2], o[2] + c[1]]);
        }
    }
    let mut s = quartet::scene(&species, &pos, false);
    s.dims = Dims::Three; s.boundary = Boundary::Walls; s.width = 17.0; s.height = 17.0; s.depth = 10.0;
    s.sync_species(); s.adopt_table_timescale(); s.thermostat_on = true; s.target_temperature = 293.0; s.rebase();
    s
}
fn main() {
    let on = std::env::args().nth(1).map(|a| a == "on").unwrap_or(true);
    let mut s = four_waters();
    if on { s.set_field(true, None).unwrap(); }
    for k in 0..=6000 {
        if k % 500 == 0 {
            let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
            let units = holon_render::field::water_units(&s.charge_row, &z).len();
            let bonded = s.pairs.iter().filter(|p| p.bonded).count();
            let (mut xmin, mut xmax) = (f64::INFINITY, f64::NEG_INFINITY);
            let mut nan = 0;
            for i in 0..s.n { let a = &s.atoms[i]; if !a.x.is_finite() { nan += 1; } xmin = xmin.min(a.x.min(a.y).min(a.z)); xmax = xmax.max(a.x.max(a.y).max(a.z)); }
            let mut min_oo = f64::INFINITY;
            for i in 0..s.n { for j in (i+1)..s.n { if z[i]==8 && z[j]==8 { let d=((s.atoms[i].x-s.atoms[j].x).powi(2)+(s.atoms[i].y-s.atoms[j].y).powi(2)+(s.atoms[i].z-s.atoms[j].z).powi(2)).sqrt(); min_oo=min_oo.min(d);} } }
            eprintln!("step {k:5}: T {:7.1} K  E {:+.6}  e_field {:+.3e}  e_pair {:+.4}  units {units}  bonded pairs {bonded}  min O-O {min_oo:.2}  coords [{xmin:.1},{xmax:.1}]  nan {nan}  transitions {}  dt {:.3}",
                s.temperature(), s.energy(), s.e_field, s.e_pair, s.field_work.transitions, s.dt());
        }
        s.step_frame(1);
    }
}
