use holon_chem::elements::{HYDROGEN, LITHIUM};
use holon_chem::pair::generate_pair_table;
fn main() {
    let t = generate_pair_table(LITHIUM, HYDROGEN, 24);
    let m = &t.meta;
    println!("LiH: worst_residual {:.4e}  worst_cg_residual {:.4e}", m.worst_residual, m.worst_cg_residual);
    println!("  residual <= bound      : {}", m.worst_residual <= holon_chem::pair::CONVERGED_RESIDUAL);
    println!("  residual finite        : {}", m.worst_residual.is_finite());
    println!("  cg residual finite     : {}", m.worst_cg_residual.is_finite());
    println!("  converged()            : {}", m.converged());
    println!("  exit                   : {}", m.exit.label());
    println!("  solve_finished()       : {}", m.solve_finished());
}
