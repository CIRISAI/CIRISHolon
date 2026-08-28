//! Unit tests for the special functions, against values this crate did not produce.
//!
//! # Where the reference values come from
//!
//! The `erf`/`erfc` column is mpmath at 60 working digits, printed to 40. The Boys `F_0`
//! column is the REFEREE's own `boys0` from `h2_core.py` — the same function the 50-digit
//! curve was built with, so a disagreement here localises to the special function rather
//! than to the integral code around it. `F_1` and `F_2` are 60-digit quadratures of their
//! defining integrals `int_0^1 u^{2m} exp(-t u^2) du`, which is an independent route to
//! them: the referee never computes `F_1` or `F_2` at all, and this crate reaches them by
//! recursion, so neither shares a step with the quadrature.
//!
//! Reference values are compared after parsing to f64, so every relative error below
//! carries a floor of half an ulp (`1.1e-16`) from that parse. The errors reported are
//! an order above it, so it is a floor rather than the measurement.

use holon_chem::dual::D2;
use holon_chem::special::{
    boys0, boys012, boys012_closed, boys012_series, boys0_d2, erf, erfc, BOYS_SERIES_MAX_T,
};

const ERF_REF: [(f64, &str, &str); 17] = [
    (0.0625, "0.07043197772238707805059005592329674391900", "0.9295680222776129219494099440767032560810"),
    (0.125, "0.1403162048013338173930294465216233981870", "0.8596837951986661826069705534783766018130"),
    (0.25, "0.2763263901682369329850682677648157120654", "0.7236736098317630670149317322351842879346"),
    (0.5, "0.5204998778130465376827466538919645287365", "0.4795001221869534623172533461080354712635"),
    (0.75, "0.7111556336535151315989378345914107773742", "0.2888443663464848684010621654085892226258"),
    (1.0, "0.8427007929497148693412206350826092592961", "0.1572992070502851306587793649173907407039"),
    (1.25, "0.9229001282564582301365234811972811404236", "0.07709987174354176986347651880271885957640"),
    (1.5, "0.9661051464753107270669762616459478586814", "0.03389485352468927293302373835405214131859"),
    (1.75, "0.9866716712191824437722111001286879766073", "0.01332832878081755622778889987131202339270"),
    (2.0, "0.9953222650189527341620692563672529286109", "0.004677734981047265837930743632747071389108"),
    (2.5, "0.9995930479825550410604357842600250872797", "0.0004069520174449589395642157399749127203487"),
    (3.0, "0.9999779095030014145586272238704176796202", "0.00002209049699858544137277612958232037984771"),
    (4.0, "0.9999999845827420997199811478403265131160", "0.00000001541725790028001885215967348688404857215"),
    (5.0, "0.9999999999984625402055719651498116565146", "0.000000000001537459794428034850188343485383378890118"),
    (6.0, "0.9999999999999999784802632875010868834066", "2.151973671249891311659335039918738463048e-17"),
    (8.0, "0.9999999999999999999999999999887757028270", "1.122429717298292707996788844317027909343e-29"),
    (10.0, "1.000000000000000000000000000000000000000", "2.088487583762544757000786294957788611561e-45"),
];


const BOYS_REF: [(f64, &str, &str, &str); 20] = [
    (0.0, "1.000000000000000000000000000000000000000", "0.3333333333333333333333333333333333333333", "0.2000000000000000000000000000000000000000"),
    (1e-12, "0.9999999999996666666666667666666666666429", "0.3333333333331333333333334047619047618862", "0.1999999999998571428571429126984126983975"),
    (1e-08, "0.9999999966666666766666666428571429034392", "0.3333333313333333404761904576719577098365", "0.1999999985714285769841269689754690075203"),
    (0.0001, "0.9999666676666428576058125301693663466309", "0.3333133340476005294793105691531604860412", "0.1999857148412546900751970197287179491593"),
    (0.01, "0.9966766429033635025688733155101723240074", "0.3313404577097724497483669165067883117684", "0.1985769690074647835597386170164188766526"),
    (0.1, "0.9676433126355918310093082711102931227419", "0.3140294729981612892252960583192825077361", "0.1862550047926214725581955775570545100686"),
    (0.5, "0.8556243918921488031733046202800450612264", "0.2490937321795153795695050852888646077845", "0.1407505368259127151047157208754133699116"),
    (1.0, "0.7468241328124270253994674361318530053545", "0.1894723458204923519019718329851960689543", "0.1002687981450173670551958643970636697086"),
    (2.0, "0.5981440066613041014657118852371713595450", "0.1157021808561728523929280975661717390343", "0.05294281483297646632119619943150770342385"),
    (5.0, "0.3957123096105135420503930524005857541508", "0.03889743626114280749537570039774373299020", "0.01099543617843429553894910527700827747217"),
    (10.0, "0.2802473905066427406353406448997051503955", "0.01401009952884401278919025266920722998926", "0.002099244932838477675801758324603056967878"),
    (20.0, "0.1981663648299736540950987941589250576981", "0.004954159069220500791413524154824616938558", "0.0003715618786626969983920686124633367664963"),
    (24.999, "0.1772489301043313988381984219376419349712", "0.003545120406624856195909481754259530609671", "0.0002127157327487645472341352069003693803782"),
    (25.0, "0.1772453850902790950764921109937813548789", "0.003544907701527823024230561807982391822657", "0.0002126944618139105041545532965857082344377"),
    (25.001, "0.1772418402889212533768398169278248116457", "0.003544695017700235800849624147713303902838", "0.0002126731938567786196510532008574801370449"),
    (30.0, "0.1618021593796400696905131930600881068380", "0.002696702656325774891013746521900649141929", "0.0001348351328147291407225472969942131517259"),
    (50.0, "0.1253314137315500251207863542266483733193", "0.001253314137315500251205934792418519815410", "0.00003759942411946500753424929392459167667927"),
    (100.0, "0.08862269254527580136490837416705725913988", "0.0004431134627263790068245418708352862956994", "0.000006646701940895685102368128062529294435491"),
    (342.0, "0.04792166376376425183506556367883568208000", "0.00007006091193532785356003737379946737146198", "0.0000003072847014707361998247253236818744362368"),
    (700.0, "0.03349622928453393862301777496869878591760", "0.00002392587806038138473072698212049913279829", "0.00000005126973870081725299441496168678385599633"),
];


fn rel(mine: f64, reference: &str) -> f64 {
    let r: f64 = reference.parse().expect("reference is a decimal");
    if r == 0.0 {
        mine.abs()
    } else {
        (mine - r).abs() / r.abs()
    }
}

#[test]
fn erf_and_erfc_against_known_values() {
    let mut worst_erf = 0.0f64;
    let mut worst_erfc = 0.0f64;
    let mut at_erf = 0.0f64;
    let mut at_erfc = 0.0f64;
    for &(x, e_ref, c_ref) in ERF_REF.iter() {
        let de = rel(erf(x), e_ref);
        let dc = rel(erfc(x), c_ref);
        if de > worst_erf {
            worst_erf = de;
            at_erf = x;
        }
        if dc > worst_erfc {
            worst_erfc = dc;
            at_erfc = x;
        }
        // The odd symmetry is a property of the function, so it is a property the
        // implementation must have exactly rather than approximately: both branches
        // route negative arguments through the same code.
        assert_eq!(erf(-x), -erf(x), "erf lost its odd symmetry at x = {x}");
    }
    println!("erf  max rel err = {worst_erf:.3e} at x = {at_erf}");
    println!("erfc max rel err = {worst_erfc:.3e} at x = {at_erfc}");
    assert!(worst_erf <= 3e-15, "erf max rel err {worst_erf:.3e}");
    // The bound grows with x because erfc carries e^{-x^2}, and rounding x^2 into an f64
    // at all costs exp a relative x^2 * eps -- 1.1e-14 at x = 10, which is where the
    // worst case sits. That is a property of f64, not of the continued fraction: no
    // implementation in this type does better.
    assert!(worst_erfc <= 3e-14, "erfc max rel err {worst_erfc:.3e}");
}

#[test]
fn erf_and_erfc_sum_to_one() {
    // Only where both are O(1) -- past x ~ 3, erfc is below f64's resolution of 1 and
    // the identity says nothing. Checking it where it is vacuous is how a broken erfc
    // passes a green test.
    let mut worst = 0.0f64;
    let mut x = -3.0f64;
    while x <= 3.0 {
        worst = worst.max((erf(x) + erfc(x) - 1.0).abs());
        x += 0.0017;
    }
    println!("max |erf + erfc - 1| on [-3, 3] = {worst:.3e}");
    assert!(worst < 1e-15);
}

#[test]
fn boys_against_reference_values() {
    let mut worst = [0.0f64; 3];
    let mut at = [0.0f64; 3];
    for &(t, r0, r1, r2) in BOYS_REF.iter() {
        let f = boys012(t);
        for (m, rf) in [r0, r1, r2].iter().enumerate() {
            let d = rel(f[m], rf);
            if d > worst[m] {
                worst[m] = d;
                at[m] = t;
            }
        }
        assert_eq!(boys0(t), f[0], "boys0 disagrees with boys012's first entry");
    }
    println!("F0 max rel err = {:.3e} at t = {}", worst[0], at[0]);
    println!("F1 max rel err = {:.3e} at t = {}", worst[1], at[1]);
    println!("F2 max rel err = {:.3e} at t = {}", worst[2], at[2]);
    for m in 0..3 {
        assert!(worst[m] <= 5e-15, "F{m} max rel err {:.3e}", worst[m]);
    }
}

#[test]
fn boys_is_ordered_and_positive_across_the_branch_change() {
    // F_m(t) = int_0^1 u^{2m} e^{-tu^2} du with u in [0,1], so the integrands are nested:
    // F_0 > F_1 > F_2 > 0 everywhere, for every t. This is the cheapest check that
    // catches the recursion running the wrong direction, and it is swept THROUGH the
    // t = 25 handover, where a discontinuity would live if the two branches disagreed.
    let mut t = 1e-9f64;
    while t < 800.0 {
        let f = boys012(t);
        assert!(f[2] > 0.0 && f[1] > f[2] && f[0] > f[1], "Boys ordering failed at t = {t}: {f:?}");
        t *= 1.0009;
    }
}

#[test]
fn the_two_boys_branches_agree_where_the_crossover_puts_them() {
    // The series and the erf-based closed form are two different computations of ONE
    // function, stitched at t = 25. The check has to evaluate them at the SAME t: a
    // "jump" measured across t = 25 - eps to t = 25 + eps is dominated by the function's
    // own slope (F0' = -F1 = -3.5e-3 there), which would pass at any crossover and did
    // pass at this one before the test was fixed.
    //
    // The sweep also SHOWS the crossover doing its job: the closed form degrades as t
    // falls (erfc(sqrt t) stops being a small correction), and the series is the accurate
    // one there, so the disagreement must grow downward and be at roundoff at 25.
    for &t in &[BOYS_SERIES_MAX_T, 30.0, 60.0, 200.0, 500.0] {
        let a = boys012_series(t);
        let b = boys012_closed(t);
        for m in 0..3 {
            let d = (a[m] - b[m]).abs() / a[m];
            println!("t = {t:>6}: F{m} series vs closed form, rel {d:.3e}");
            assert!(
                d < 5e-15,
                "the two Boys branches disagree by {d:.3e} at t = {t}, m = {m}"
            );
        }
    }
    // Below the crossover the closed form is the WORSE route, which is the reason the
    // crossover exists. If this ever stops holding, the crossover is in the wrong place.
    let low = 1e-3;
    let degraded = (boys012_series(low)[0] - boys012_closed(low)[0]).abs() / boys012_series(low)[0];
    println!("t = {low}: the closed form is off by {degraded:.3e} -- the series' territory");
    assert!(
        degraded > 1e-12,
        "the closed form did NOT degrade at small t, so the crossover is not buying \
         anything and its placement needs re-deriving"
    );
}

#[test]
fn boys_derivative_rules_hold() {
    // dF0/dt = -F1 and d2F0/dt2 = +F2, from differentiating under the integral sign.
    // Checked against a five-point central difference of F0 itself -- which is a WEAK
    // instrument (its own noise is ~1e-10) and is used as one: it cannot certify the
    // derivative, only catch a rule attached to the wrong function or the wrong sign.
    for &t0 in &[0.05f64, 0.7, 3.0, 12.0, 24.0, 40.0, 200.0] {
        let h = 1e-3 * t0.max(1.0);
        let f = |x: f64| boys0(x);
        let d1 = (f(t0 - 2.0 * h) - 8.0 * f(t0 - h) + 8.0 * f(t0 + h) - f(t0 + 2.0 * h))
            / (12.0 * h);
        let d2 = (-f(t0 - 2.0 * h) + 16.0 * f(t0 - h) - 30.0 * f(t0) + 16.0 * f(t0 + h)
            - f(t0 + 2.0 * h))
            / (12.0 * h * h);
        let out = boys0_d2(D2::var(t0));
        let e1 = (out.d - d1).abs() / d1.abs();
        let e2 = (out.e - d2).abs() / d2.abs();
        println!("t = {t0:>6}: dF0 rel {e1:.2e}, d2F0 rel {e2:.2e}");
        assert!(e1 < 1e-8, "dF0/dt disagrees with the difference quotient at t = {t0}");
        assert!(e2 < 1e-5, "d2F0/dt2 disagrees with the difference quotient at t = {t0}");
    }
}

#[test]
fn boys_at_zero_is_the_exact_rational() {
    // F_m(0) = int_0^1 u^{2m} du = 1/(2m+1). Exact rationals, so this is an equality
    // test rather than a tolerance test, and it pins the series' first term.
    let f = boys012(0.0);
    assert_eq!(f[0], 1.0);
    assert!((f[1] - 1.0 / 3.0).abs() <= f64::EPSILON);
    assert!((f[2] - 1.0 / 5.0).abs() <= f64::EPSILON);
}

#[test]
fn the_hoisted_constants_still_equal_the_expressions_they_replace() {
    // Two transcendentals are written as literals because they sit in the innermost loop
    // and `powf` was being asked to recompute them hundreds of times per knot. A literal
    // that has drifted from its expression is the quietest possible bug: everything still
    // computes, and every answer is slightly wrong. Pinning them costs one test.
    assert_eq!(
        holon_chem::sto3g::PI_POW_2_5,
        std::f64::consts::PI.powf(2.5),
        "PI_POW_2_5 is not pi^2.5 on this platform"
    );
    assert_eq!(
        holon_chem::special::SQRT_PI_FOR_TEST,
        std::f64::consts::PI.sqrt(),
        "SQRT_PI is not sqrt(pi) on this platform"
    );
}
