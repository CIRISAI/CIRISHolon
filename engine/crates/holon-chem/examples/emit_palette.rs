//! Emit the sandbox's periodic palette: one entry per first-row element, with the drawn
//! radius DERIVED from that element's own computed homonuclear curve and the colour from
//! the DECLARED rule. Writes `species_palette.json` to `$PALETTE_OUT` (default `.`).
use holon_chem::elements::FIRST_ROW;
use holon_chem::pair::{
    declared_colour, homonuclear_size, RadiusRule, CONTACT_ENERGY, PAIR_PROVENANCE, WELL_MIN_DEPTH,
};

fn main() {
    // Default under `target/`, which is gitignored, NOT the working directory. Defaulting
    // to "." put nine untracked result files in the repository root the first time
    // somebody ran this without the variable set, where they sat looking like committed
    // artifacts and carried a superseded basis. An example that writes results should not
    // be able to litter the tree by being run the obvious way.
    let out = std::env::var("PALETTE_OUT").unwrap_or_else(|_| "target/elements1".into());
    std::fs::create_dir_all(&out).expect("output directory");
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"provenance\": \"{PAIR_PROVENANCE}\",\n"));
    s.push_str(
        "  \"note\": \"radius_bohr is DERIVED: half the element's own computed homonuclear \
         separation -- the equilibrium separation where that pair binds, and the declared \
         contact separation where it does not. colour is DECLARED: a monotone hue ramp in Z. \
         Both rules are stated in holon-chem/src/pair.rs.\",\n",
    );
    s.push_str(&format!("  \"well_min_depth_hartree\": {WELL_MIN_DEPTH:?},\n"));
    s.push_str(&format!("  \"contact_energy_hartree\": {CONTACT_ENERGY:?},\n"));
    s.push_str("  \"species\": [\n");
    for (i, sp) in FIRST_ROW.iter().enumerate() {
        let sz = homonuclear_size(*sp);
        eprintln!(
            "  {:>2} {:>3}  radius {:.4} a0  rule {:?}  sep {:.4}  D_e {:?}  ndet {}  {:.1} s",
            sp.z, sp.symbol, sz.radius_bohr, sz.rule, sz.separation_bohr, sz.d_e, sz.n_det,
            sz.ms / 1e3
        );
        s.push_str("    {");
        s.push_str(&format!("\"symbol\": \"{}\", ", sp.symbol));
        s.push_str(&format!("\"Z\": {}, ", sp.z));
        s.push_str(&format!("\"mass_me\": {:?}, ", sp.mass_me()));
        s.push_str(&format!("\"mass_u\": {:?}, ", sp.mass_u));
        s.push_str(&format!("\"isotope\": \"{}\", ", sp.isotope));
        s.push_str(&format!("\"n_basis\": {}, ", sp.n_basis()));
        s.push_str(&format!("\"homonuclear_n_determinants\": {}, ", sz.n_det));
        s.push_str(&format!("\"radius_bohr\": {:?}, ", sz.radius_bohr));
        // The wording comes from the rule itself, not from a match written here. Two
        // surfaces describing one distinction in their own words is how they come to
        // disagree about it, and this distinction is the one P1's plant guards.
        s.push_str(&format!("\"radius_rule\": \"{}\", ", sz.rule.describe()));
        // The distinction a consumer must be able to read WITHOUT parsing prose: a
        // dimer-derived radius is half a measured inter-atomic separation, a
        // density-derived one is one atom's own extent, and they are not the same axis.
        s.push_str(&format!(
            "\"radius_from_dimer\": {}, ",
            sz.rule.is_dimer_derived()
        ));
        // NaN is not JSON. A density-derived radius has no separation at all, and `null`
        // says that; a zero would read as a measurement of a curve that never ran.
        if sz.separation_bohr.is_finite() {
            s.push_str(&format!(
                "\"homonuclear_separation_bohr\": {:?}, ",
                sz.separation_bohr
            ));
        } else {
            s.push_str("\"homonuclear_separation_bohr\": null, ");
        }
        match sz.d_e {
            Some(d) => s.push_str(&format!("\"homonuclear_bound\": true, \"homonuclear_D_e\": {d:?}, ")),
            None => s.push_str("\"homonuclear_bound\": false, \"homonuclear_D_e\": null, "),
        }
        s.push_str(&format!("\"colour\": \"{}\", ", declared_colour(sp.z)));
        s.push_str("\"colour_rule\": \"declared: monotone hue ramp in Z\"}");
        if i + 1 < FIRST_ROW.len() {
            s.push(',');
        }
        s.push('\n');
    }
    s.push_str("  ]\n}\n");
    let path = format!("{out}/species_palette.json");
    std::fs::write(&path, s).unwrap();
    eprintln!("wrote {path}");
}
