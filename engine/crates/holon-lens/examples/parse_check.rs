//! Can the quench-log parser read the full-strength dE4 log? Checked BEFORE the
//! adjudication needs it, so a format surprise is not discovered at verdict time.
use holon_lens::quenchlog::QuenchLog;
fn main() {
    for p in std::env::args().skip(1) {
        let t = std::fs::read_to_string(&p).expect("readable");
        let log = QuenchLog::parse(&t);
        println!("# {}", p.rsplit('/').next().unwrap());
        println!("#   header surfaces : {:?}", log.header_tables);
        println!("#   seed rows       : {}", log.seeds.len());
        for s in &log.seeds {
            println!(
                "#   seed {:#018x}  modal-O {:?}  molecules {:?}  fenced {}",
                s.seed, s.modal_o, s.molecules, s.fenced
            );
        }
        println!("#   OH2 as a MOLECULE: {}", log.molecule_count("OH2"));
    }
}
