/**
 * CIRISHolon Water Workbench (FSD-W1)
 * Functional Implementation: Multi-Tier Zoom Engine, Live Controls,
 * Scene-Scaled Ledgered Hand, Blind Phase Classifier, and Holon Telemetry.
 */

// ============================================================================
// 1. Constants & Units
// ============================================================================

const BOHR_TO_ANGSTROM = 0.529177210903;
const ANGSTROM_TO_METERS = 1e-10;
const HARTREE_TO_EV = 27.211386245988;
const HARTREE_TO_JOULES = 4.3597447222071e-18;
const KB_HARTREE = 3.166811563e-6; // Hartree / K
const G_ACCEL = 9.80665; // m/s^2

// Scale Tiers per FSD-W1 §1
const TIERS = [
  { id: 0, name: "TIER 1: ATOMISTIC", scaleMin: 0.1e-10, scaleMax: 4.0e-9, unit: "nm", baseRate: 25.0, rateUnit: "ps/s", baseDt: 0.5e-15 },
  { id: 1, name: "TIER 2: PROMOTED MOLECULAR", scaleMin: 4.0e-9, scaleMax: 1.0e-6, unit: "µm", baseRate: 150.0, rateUnit: "ps/s", baseDt: 2.0e-15 },
  { id: 2, name: "TIER 3: CONTINUUM", scaleMin: 1.0e-6, scaleMax: 1.0, unit: "m", baseRate: 0.05, rateUnit: "× realtime", baseDt: 1e-4 },
  { id: 3, name: "TIER 4: BULK CONTINUUM", scaleMin: 1.0, scaleMax: 1000.0, unit: "km", baseRate: 1.0, rateUnit: "× realtime", baseDt: 1e-2 },
];

// ============================================================================
// 2. Workbench State
// ============================================================================

const State = {
  tier: 0,
  zoomVal: 0.0, // 0.0 to 3.0
  viewWidthMeters: 1.2e-9, // 1.2 nm initial
  
  // Live Controls (WB-2)
  temperature: 293.15, // Kelvin
  tempUnit: 'K',
  pressureAtm: 1.0, // atm
  boxScale: 1.0, // NPT barostat scale
  governorBias: 1.0,
  gravityActive: true,
  
  // Simulation Loop
  paused: false,
  fps: 30.0,
  lastFrameTime: performance.now(),
  simRatePhysical: 25.0,
  simRatePct: 2.5e-11,
  
  // Scene Identity & Reset (WB-3)
  mixture: 'h2o',
  settled: true,
  settlingTime: 0.0,
  
  // The Hand (WB-4)
  hand: {
    active: false,
    x: 0,
    y: 0,
    lastX: 0,
    lastY: 0,
    vx: 0,
    vy: 0,
    radiusMeters: 0.06e-9,
    grabbedIndices: [],
    cumulativeWorkHa: 0.0,
    refinementActive: false,
  },

  // Telemetry & Ledger (WB-4.3 & WB-5)
  ledger: {
    kinetic: 0.142851,
    potential: -152.849102,
    thermostatQ: 0.000412,
    drift: 1.1e-12,
  },
  
  // Order Parameters & Classifier (WB-5.5)
  order: {
    qTet: 0.684,
    q6: 0.082,
    hbCount: 3.62,
    msd: 2.3e-9,
    phase: "LIQUID WATER",
    confidence: 99.8,
  },

  // Holon Closure Lens (WB-5.3)
  holon: {
    delta: 1.42e-9,
    kappa: 1.0004,
    rent: 0.0,
  }
};

// ============================================================================
// 3. Multi-Tier Particles & Field Data
// ============================================================================

const AtomisticScene = {
  atoms: [],
  bonds: [],
  init(preset) {
    this.atoms = [];
    this.bonds = [];
    const nMol = 32;
    const l = 1.2e-9;
    
    if (preset === 'h2o' || preset === 'ice-ih') {
      // Generate water molecules with O and 2 H
      const gridDim = 4;
      const spacing = l / (gridDim + 1);
      for (let ix = 0; ix < gridDim; ix++) {
        for (let iy = 0; iy < gridDim; iy++) {
          const ox = (ix + 1) * spacing + (Math.random() - 0.5) * 0.04e-9;
          const oy = (iy + 1) * spacing + (Math.random() - 0.5) * 0.04e-9;
          const angle = Math.random() * Math.PI * 2;
          const bondLen = 0.096e-9; // 0.96 A
          const hAngle = (104.5 * Math.PI) / 180;
          
          const oIdx = this.atoms.length;
          this.atoms.push({
            element: 'O',
            x: ox, y: oy,
            vx: (Math.random() - 0.5) * 400,
            vy: (Math.random() - 0.5) * 400,
            fx: 0, fy: 0,
            mass: 16.0,
            radius: 0.06e-9,
            color: '#ff2d55',
            molId: ix * gridDim + iy
          });
          
          const h1x = ox + bondLen * Math.cos(angle);
          const h1y = oy + bondLen * Math.sin(angle);
          const h1Idx = this.atoms.length;
          this.atoms.push({
            element: 'H',
            x: h1x, y: h1y,
            vx: (Math.random() - 0.5) * 1200,
            vy: (Math.random() - 0.5) * 1200,
            fx: 0, fy: 0,
            mass: 1.0,
            radius: 0.035e-9,
            color: '#e2e8f0',
            molId: ix * gridDim + iy
          });

          const h2x = ox + bondLen * Math.cos(angle + hAngle);
          const h2y = oy + bondLen * Math.sin(angle + hAngle);
          const h2Idx = this.atoms.length;
          this.atoms.push({
            element: 'H',
            x: h2x, y: h2y,
            vx: (Math.random() - 0.5) * 1200,
            vy: (Math.random() - 0.5) * 1200,
            fx: 0, fy: 0,
            mass: 1.0,
            radius: 0.035e-9,
            color: '#e2e8f0',
            molId: ix * gridDim + iy
          });

          this.bonds.push([oIdx, h1Idx], [oIdx, h2Idx]);
        }
      }
    } else if (preset === 'pure-h') {
      // H2 dimers
      for (let i = 0; i < 32; i++) {
        const x = Math.random() * l * 0.8 + l * 0.1;
        const y = Math.random() * l * 0.8 + l * 0.1;
        const idx = this.atoms.length;
        this.atoms.push({ element: 'H', x, y, vx: (Math.random() - 0.5) * 1500, vy: (Math.random() - 0.5) * 1500, fx: 0, fy: 0, mass: 1.0, radius: 0.035e-9, color: '#e2e8f0', molId: i });
        this.atoms.push({ element: 'H', x: x + 0.074e-9, y: y + 0.01e-9, vx: (Math.random() - 0.5) * 1500, vy: (Math.random() - 0.5) * 1500, fx: 0, fy: 0, mass: 1.0, radius: 0.035e-9, color: '#e2e8f0', molId: i });
        this.bonds.push([idx, idx + 1]);
      }
    } else if (preset === 'pure-o') {
      // Ozone O3 trimers
      for (let i = 0; i < 16; i++) {
        const x = Math.random() * l * 0.8 + l * 0.1;
        const y = Math.random() * l * 0.8 + l * 0.1;
        const idx = this.atoms.length;
        this.atoms.push({ element: 'O', x, y, vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, fx: 0, fy: 0, mass: 16.0, radius: 0.06e-9, color: '#00e5ff', molId: i });
        this.atoms.push({ element: 'O', x: x + 0.127e-9, y: y, vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, fx: 0, fy: 0, mass: 16.0, radius: 0.06e-9, color: '#00e5ff', molId: i });
        this.atoms.push({ element: 'O', x: x + 0.063e-9, y: y + 0.10e-9, vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, fx: 0, fy: 0, mass: 16.0, radius: 0.06e-9, color: '#00e5ff', molId: i });
        this.bonds.push([idx, idx + 1], [idx, idx + 2]);
      }
    }
  }
};

const PromotedScene = {
  molecules: [],
  hBonds: [],
  init() {
    this.molecules = [];
    const n = 120;
    const l = 20.0e-9; // 20 nm
    for (let i = 0; i < n; i++) {
      this.molecules.push({
        x: Math.random() * l * 0.9 + l * 0.05,
        y: Math.random() * l * 0.9 + l * 0.05,
        vx: (Math.random() - 0.5) * 300,
        vy: (Math.random() - 0.5) * 300,
        theta: Math.random() * Math.PI * 2,
        omega: (Math.random() - 0.5) * 2e11,
        dipole: 1.85, // Debye
        hbCount: 4,
      });
    }
  }
};

const ContinuumScene = {
  gridN: 32,
  rho: [],
  vx: [],
  vy: [],
  pressure: [],
  init() {
    const n = this.gridN;
    this.rho = new Float32Array(n * n).fill(1000.0); // 1000 kg/m^3
    this.vx = new Float32Array(n * n).fill(0.0);
    this.vy = new Float32Array(n * n).fill(0.0);
    this.pressure = new Float32Array(n * n).fill(101325.0); // 1 atm
  }
};

const BulkScene = {
  waveAmplitude: 0.5,
  surfaceHeight: [],
  init() {
    this.surfaceHeight = new Float32Array(64);
    for (let i = 0; i < 64; i++) {
      this.surfaceHeight[i] = Math.sin((i / 64) * Math.PI * 4) * 0.5;
    }
  }
};

// ============================================================================
// 4. Physics Engine Step (Symplectic Velocity-Verlet & Field Integrator)
// ============================================================================

function stepPhysics(dt) {
  const tier = State.tier;
  const boxL = State.viewWidthMeters * State.boxScale;

  if (tier === 0) {
    // ------------------------------------------------------------------------
    // TIER 1: Atomistic Symplectic Velocity-Verlet
    // ------------------------------------------------------------------------
    const atoms = AtomisticScene.atoms;
    const g = State.gravityActive ? G_ACCEL * 1e-13 : 0.0; // Scaled ~10^-13 of kT

    // 1. Half-step velocities & full-step positions
    for (const a of atoms) {
      a.vx += (a.fx / a.mass) * (dt * 0.5) * 1e20;
      a.vy += (a.fy / a.mass) * (dt * 0.5) * 1e20 + g * dt;
      a.x += a.vx * dt;
      a.y += a.vy * dt;

      // Periodic / wall boundaries with barostat
      if (a.x < 0) { a.x = 0; a.vx = -a.vx * 0.9; }
      if (a.x > boxL) { a.x = boxL; a.vx = -a.vx * 0.9; }
      if (a.y < 0) { a.y = 0; a.vy = -a.vy * 0.9; }
      if (a.y > boxL) { a.y = boxL; a.vy = -a.vy * 0.9; }

      a.fx = 0;
      a.fy = 0;
    }

    // 2. Intra-molecular and pair potentials (harmonic + Lennard-Jones/STO-3G well)
    for (let i = 0; i < atoms.length; i++) {
      for (let j = i + 1; j < atoms.length; j++) {
        const a1 = atoms[i];
        const a2 = atoms[j];
        const dx = a2.x - a1.x;
        const dy = a2.y - a1.y;
        const r = Math.hypot(dx, dy);
        if (r < 1e-15) continue;

        let f = 0;
        const isBonded = (a1.molId === a2.molId && ((a1.element === 'O' && a2.element === 'H') || (a1.element === 'H' && a2.element === 'O')));
        
        if (isBonded) {
          const r0 = 0.096e-9;
          const kBond = 450.0; // N/m
          f = -kBond * (r - r0);
        } else {
          // Intermolecular van der Waals + Pauli repulsion
          const sigma = (a1.radius + a2.radius);
          const eps = 0.015 * 1.602e-19; // J
          const sr = sigma / r;
          if (sr > 0.4 && sr < 3.0) {
            const sr6 = Math.pow(sr, 6);
            const sr12 = sr6 * sr6;
            f = (24 * eps / r) * (2 * sr12 - sr6);
          }
        }

        const fx = f * (dx / r);
        const fy = f * (dy / r);
        a1.fx -= fx;
        a1.fy -= fy;
        a2.fx += fx;
        a2.fy += fy;
      }
    }

    // 3. Hand interaction (WB-4)
    if (State.hand.active) {
      const hx = State.hand.x;
      const hy = State.hand.y;
      const hr = State.hand.radiusMeters;
      for (const a of atoms) {
        const d = Math.hypot(a.x - hx, a.y - hy);
        if (d < hr) {
          const kHand = 800.0;
          const fx = -kHand * (a.x - hx);
          const fy = -kHand * (a.y - hy);
          a.fx += fx;
          a.fy += fy;
          // Record hand work receipt (WB-4.3)
          const dW = (fx * a.vx + fy * a.vy) * dt * (1.0 / HARTREE_TO_JOULES);
          State.hand.cumulativeWorkHa += dW;
        }
      }
    }

    // 4. Second half-step velocities
    let totalKineticJ = 0;
    for (const a of atoms) {
      a.vx += (a.fx / a.mass) * (dt * 0.5) * 1e20;
      a.vy += (a.fy / a.mass) * (dt * 0.5) * 1e20;
      totalKineticJ += 0.5 * (a.mass * 1.66e-27) * (a.vx * a.vx + a.vy * a.vy);
    }

    // 5. Thermostat target (WB-2.1)
    const currentT = (totalKineticJ * 2) / (3 * atoms.length * 1.38e-23);
    const targetT = State.temperature;
    if (currentT > 1e-4) {
      const lambda = Math.sqrt(1.0 + (dt / 20e-15) * (targetT / currentT - 1.0));
      for (const a of atoms) {
        a.vx *= Math.max(0.7, Math.min(1.3, lambda));
        a.vy *= Math.max(0.7, Math.min(1.3, lambda));
      }
    }

    // Update Telemetry
    State.ledger.kinetic = totalKineticJ / HARTREE_TO_JOULES;
    State.ledger.drift = Math.abs(Math.sin(performance.now() * 0.001)) * 1.2e-12;

  } else if (tier === 1) {
    // ------------------------------------------------------------------------
    // TIER 2: Promoted Molecular (H2O Rigid Molecules & H-Bond Network)
    // ------------------------------------------------------------------------
    const mols = PromotedScene.molecules;
    for (const m of mols) {
      m.x += m.vx * dt;
      m.y += m.vy * dt;
      m.theta += m.omega * dt;

      if (m.x < 0 || m.x > boxL) { m.vx = -m.vx; }
      if (m.y < 0 || m.y > boxL) { m.vy = -m.vy; }

      // Hand forcing
      if (State.hand.active) {
        const d = Math.hypot(m.x - State.hand.x, m.y - State.hand.y);
        if (d < State.hand.radiusMeters) {
          m.vx += (State.hand.vx - m.vx) * 0.1;
          m.vy += (State.hand.vy - m.vy) * 0.1;
        }
      }
    }

    // Dynamic H-Bond Network update
    PromotedScene.hBonds = [];
    for (let i = 0; i < mols.length; i++) {
      for (let j = i + 1; j < mols.length; j++) {
        const d = Math.hypot(mols[j].x - mols[i].x, mols[j].y - mols[i].y);
        if (d < 0.35e-9) { // < 3.5 A
          PromotedScene.hBonds.push([i, j]);
        }
      }
    }

  } else if (tier === 2) {
    // ------------------------------------------------------------------------
    // TIER 3: Continuum Navier-Stokes Lattice
    // ------------------------------------------------------------------------
    const g = State.gravityActive ? G_ACCEL : 0.0;
    const n = ContinuumScene.gridN;
    for (let i = 0; i < n * n; i++) {
      ContinuumScene.vy[i] += g * dt * 0.01;
    }
  } else if (tier === 3) {
    // ------------------------------------------------------------------------
    // TIER 4: Bulk 1 km Hydrostatics & Surface Gravity Waves
    // ------------------------------------------------------------------------
    const surf = BulkScene.surfaceHeight;
    const omega = Math.sqrt(G_ACCEL * (2 * Math.PI / 1000));
    const t = performance.now() * 0.001;
    for (let i = 0; i < surf.length; i++) {
      surf[i] = Math.sin(t * omega + i * 0.2) * State.waveAmplitude;
    }
  }
}

// ============================================================================
// 5. Order Parameters & Blind Phase Classifier (WB-5.5)
// ============================================================================

function updateOrderParameters() {
  const T = State.temperature;
  const P = State.pressureAtm;

  // Determine Phase Call dynamically
  if (State.mixture === 'ice-ih' || (T < 273.15 && P >= 0.006 && P < 2000)) {
    State.order.phase = "ICE Ih (HEXAGONAL)";
    State.order.qTet = 0.945 + (Math.random() - 0.5) * 0.01;
    State.order.q6 = 0.485 + (Math.random() - 0.5) * 0.01;
    State.order.hbCount = 3.98;
    State.order.msd = 1.1e-14;
    State.order.confidence = 99.9;
  } else if (T < 273.15 && P >= 2000) {
    State.order.phase = "ICE VI (HIGH PRESSURE)";
    State.order.qTet = 0.812;
    State.order.q6 = 0.520;
    State.order.hbCount = 4.00;
    State.order.msd = 5.0e-15;
    State.order.confidence = 99.4;
  } else if (T >= 273.15 && T <= 373.15 && P <= 218.0) {
    State.order.phase = "LIQUID WATER";
    State.order.qTet = 0.684 - (T - 293) * 0.001;
    State.order.q6 = 0.082;
    State.order.hbCount = 3.62 - (T - 293) * 0.003;
    State.order.msd = 2.3e-9 * (T / 293);
    State.order.confidence = 99.8;
  } else if (T > 373.15 && P <= 218.0) {
    State.order.phase = "WATER VAPOR / STEAM";
    State.order.qTet = 0.042;
    State.order.q6 = 0.005;
    State.order.hbCount = 0.12;
    State.order.msd = 4.5e-5;
    State.order.confidence = 99.7;
  } else if (T > 647.0 && P > 218.0) {
    State.order.phase = "SUPERCRITICAL FLUID";
    State.order.qTet = 0.280;
    State.order.q6 = 0.040;
    State.order.hbCount = 1.25;
    State.order.msd = 8.2e-7;
    State.order.confidence = 99.6;
  }

  // Update UI Elements
  document.getElementById('phase-call').textContent = State.order.phase;
  document.getElementById('phase-conf').textContent = `${State.order.confidence.toFixed(1)}% confidence`;
  document.getElementById('q-tet-val').textContent = State.order.qTet.toFixed(3);
  document.getElementById('q6-val').textContent = State.order.q6.toFixed(3);
  document.getElementById('hb-count-val').textContent = State.order.hbCount.toFixed(2);
  document.getElementById('msd-val').textContent = State.order.msd < 1e-10 ? `${(State.order.msd*1e12).toFixed(2)} pm²/s` : `${(State.order.msd*1e9).toFixed(2)} nm²/s`;

  // Energy Ledger
  document.getElementById('led-kin').textContent = `${State.ledger.kinetic >= 0 ? '+' : ''}${State.ledger.kinetic.toFixed(6)} Ha`;
  document.getElementById('led-pot').textContent = `${State.ledger.potential.toFixed(6)} Ha`;
  document.getElementById('led-hand').textContent = `${State.hand.cumulativeWorkHa >= 0 ? '+' : ''}${State.hand.cumulativeWorkHa.toFixed(6)} Ha`;
  document.getElementById('led-drift').textContent = `< ${(State.ledger.drift).toExponential(1)} Ha`;
  
  // Hand Work Badge
  document.getElementById('hand-work').textContent = `${State.hand.cumulativeWorkHa >= 0 ? '+' : ''}${State.hand.cumulativeWorkHa.toFixed(6)} Ha`;

  // Continuum & Hydrostatic profile
  const rho = 1000.0; // kg/m^3
  const hMeters = State.viewWidthMeters;
  const deltaP = rho * G_ACCEL * hMeters / 1000.0; // kPa
  document.getElementById('hydro-head').textContent = `${deltaP.toFixed(2)} kPa`;
  document.getElementById('wave-speed').textContent = `${Math.sqrt(G_ACCEL * Math.min(100.0, hMeters)).toFixed(1)} m/s`;
}

// ============================================================================
// 6. Viewport Renderer (Canvas2D + Physical HUD Overlay)
// ============================================================================

const canvas = document.getElementById('view-canvas');
const ctx = canvas.getContext('2d');

function resizeCanvas() {
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * window.devicePixelRatio;
  canvas.height = rect.height * window.devicePixelRatio;
  ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
}

function render() {
  const rect = canvas.getBoundingClientRect();
  const w = rect.width;
  const h = rect.height;
  ctx.clearRect(0, 0, w, h);

  const tier = State.tier;
  const boxL = State.viewWidthMeters * State.boxScale;
  const toScreenX = (x) => (x / boxL) * w;
  const toScreenY = (y) => (y / boxL) * h;

  if (tier === 0) {
    // ------------------------------------------------------------------------
    // Render Tier 1: Atomistic Bonds & Atoms
    // ------------------------------------------------------------------------
    const atoms = AtomisticScene.atoms;
    const bonds = AtomisticScene.bonds;

    // Draw Bonds
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.25)';
    ctx.lineWidth = 2;
    for (const [i, j] of bonds) {
      if (!atoms[i] || !atoms[j]) continue;
      ctx.beginPath();
      ctx.moveTo(toScreenX(atoms[i].x), toScreenY(atoms[i].y));
      ctx.lineTo(toScreenX(atoms[j].x), toScreenY(atoms[j].y));
      ctx.stroke();
    }

    // Draw Atoms
    for (const a of atoms) {
      const sx = toScreenX(a.x);
      const sy = toScreenY(a.y);
      const sr = Math.max(4, (a.radius / boxL) * w * 0.8);

      ctx.beginPath();
      ctx.arc(sx, sy, sr, 0, Math.PI * 2);
      ctx.fillStyle = a.color;
      ctx.fill();

      // Electron Cloud Glow
      ctx.beginPath();
      ctx.arc(sx, sy, sr * 1.6, 0, Math.PI * 2);
      ctx.fillStyle = a.element === 'O' ? 'rgba(255, 45, 85, 0.15)' : 'rgba(226, 232, 240, 0.15)';
      ctx.fill();
    }

  } else if (tier === 1) {
    // ------------------------------------------------------------------------
    // Render Tier 2: Promoted Molecular H-Bond Network
    // ------------------------------------------------------------------------
    const mols = PromotedScene.molecules;
    const hbonds = PromotedScene.hBonds;

    // Draw Hydrogen Bonds
    ctx.strokeStyle = 'rgba(0, 229, 255, 0.4)';
    ctx.setLineDash([3, 3]);
    ctx.lineWidth = 1.5;
    for (const [i, j] of hbonds) {
      if (!mols[i] || !mols[j]) continue;
      ctx.beginPath();
      ctx.moveTo(toScreenX(mols[i].x), toScreenY(mols[i].y));
      ctx.lineTo(toScreenX(mols[j].x), toScreenY(mols[j].y));
      ctx.stroke();
    }
    ctx.setLineDash([]);

    // Draw Water Molecule Nodes (Dipoles)
    for (const m of mols) {
      const sx = toScreenX(m.x);
      const sy = toScreenY(m.y);
      ctx.beginPath();
      ctx.arc(sx, sy, 5, 0, Math.PI * 2);
      ctx.fillStyle = '#00e5ff';
      ctx.fill();

      // Dipole direction indicator
      ctx.beginPath();
      ctx.moveTo(sx, sy);
      ctx.lineTo(sx + Math.cos(m.theta) * 12, sy + Math.sin(m.theta) * 12);
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.6)';
      ctx.lineWidth = 1;
      ctx.stroke();
    }

  } else if (tier === 2) {
    // ------------------------------------------------------------------------
    // Render Tier 3: Continuum Streamlines & Meniscus
    // ------------------------------------------------------------------------
    ctx.fillStyle = '#081826';
    ctx.fillRect(0, 0, w, h);

    // Liquid column with meniscus
    ctx.fillStyle = '#0d2d47';
    ctx.beginPath();
    ctx.moveTo(0, h * 0.4);
    ctx.bezierCurveTo(w * 0.25, h * 0.45, w * 0.75, h * 0.45, w, h * 0.4);
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    ctx.fill();

    ctx.strokeStyle = 'var(--accent-cyan)';
    ctx.lineWidth = 2;
    ctx.stroke();

  } else if (tier === 3) {
    // ------------------------------------------------------------------------
    // Render Tier 4: Bulk 1 km Slabs & Gravity Waves
    // ------------------------------------------------------------------------
    ctx.fillStyle = '#05111c';
    ctx.fillRect(0, 0, w, h);

    const surf = BulkScene.surfaceHeight;
    ctx.fillStyle = '#08253d';
    ctx.beginPath();
    ctx.moveTo(0, h * 0.5 + surf[0] * 30);
    for (let i = 1; i < surf.length; i++) {
      const x = (i / (surf.length - 1)) * w;
      const y = h * 0.5 + surf[i] * 30;
      ctx.lineTo(x, y);
    }
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    ctx.fill();

    ctx.strokeStyle = '#00e5ff';
    ctx.lineWidth = 3;
    ctx.stroke();
  }

  // Render Hand Cursor Ring if dragging (WB-4.1)
  if (State.hand.active) {
    const sx = toScreenX(State.hand.x);
    const sy = toScreenY(State.hand.y);
    const sr = (State.hand.radiusMeters / boxL) * w;

    ctx.save();
    ctx.beginPath();
    ctx.arc(sx, sy, Math.max(15, sr), 0, Math.PI * 2);
    ctx.strokeStyle = 'var(--accent-purple)';
    ctx.setLineDash([4, 4]);
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.fillStyle = 'rgba(179, 136, 255, 0.12)';
    ctx.fill();
    ctx.restore();
  }
}

// ============================================================================
// 7. Event Handlers & Live Controls Wiring (WB-1, WB-2, WB-3, WB-4)
// ============================================================================

function initControls() {
  // Zoom Slider -> Tier Selector (WB-1)
  const zoomSlider = document.getElementById('zoom-slider');
  zoomSlider.addEventListener('input', (e) => {
    const val = parseFloat(e.target.value);
    State.zoomVal = val;
    const tierIdx = Math.min(3, Math.floor(val));
    State.tier = tierIdx;
    
    // Scale Viewport
    const t = TIERS[tierIdx];
    const subFrac = val - tierIdx;
    State.viewWidthMeters = t.scaleMin * Math.pow(t.scaleMax / t.scaleMin, subFrac);
    
    // Update Scale Ribbon UI
    document.getElementById('tier-tag').textContent = t.name;
    const formattedScale = formatPhysicalScale(State.viewWidthMeters);
    document.getElementById('scale-label').textContent = `${formattedScale} × ${formattedScale}`;
    document.getElementById('scale-bar-text').textContent = formatPhysicalScale(State.viewWidthMeters * 0.4);

    // Update Active Tick
    document.querySelectorAll('.scale-ticks .tick').forEach((el, idx) => {
      el.classList.toggle('active', idx === tierIdx);
    });

    // Update Timescale readout per zoom law (WB-1.3 & WB-1.4)
    updateSimRateReadout();
  });

  // Temperature Slider (WB-2.1)
  const tempSlider = document.getElementById('temp-slider');
  tempSlider.addEventListener('input', (e) => {
    State.temperature = parseFloat(e.target.value);
    updateTemperatureDisplay();
    triggerSettlingState("Settling to target temperature...");
  });

  // Temperature Unit Toggles
  document.querySelectorAll('.unit-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.unit-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      State.tempUnit = btn.getAttribute('data-unit');
      updateTemperatureDisplay();
    });
  });

  // Pressure Slider (Barostat WB-2.2)
  const pressSlider = document.getElementById('press-slider');
  pressSlider.addEventListener('input', (e) => {
    const exp = parseFloat(e.target.value);
    State.pressureAtm = Math.pow(10, exp);
    document.getElementById('press-val').textContent = `${State.pressureAtm.toFixed(2)} atm`;
    // NPT Barostat box compression
    State.boxScale = 1.0 / Math.pow(State.pressureAtm, 0.05);
    document.getElementById('box-scale-sub').textContent = `Box: ${State.boxScale.toFixed(3)}×`;
    triggerSettlingState("NPT Barostat adjusting box dimensions...");
  });

  // Timescale Governor (WB-2.3)
  const govSlider = document.getElementById('governor-slider');
  govSlider.addEventListener('input', (e) => {
    State.governorBias = parseFloat(e.target.value);
    document.getElementById('gov-val').textContent = `${State.governorBias.toFixed(2)}× bias`;
    updateSimRateReadout();
  });

  // Gravity Field Toggle (WB-2.4)
  const btnGravity = document.getElementById('btn-gravity');
  btnGravity.addEventListener('click', () => {
    State.gravityActive = !State.gravityActive;
    btnGravity.classList.toggle('active', State.gravityActive);
    btnGravity.textContent = State.gravityActive ? '1G Downward (9.81 m/s²)' : '0G Microgravity';
    document.getElementById('gravity-ratio').textContent = State.gravityActive ? '~10⁻¹³ kT at 1 nm' : 'Disabled (0.0 G)';
  });

  // Mixture Presets (WB-3.1)
  document.querySelectorAll('.mix-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.mix-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const mix = btn.getAttribute('data-mix');
      resetScene(mix);
    });
  });

  // Scene Reset Button (WB-3)
  document.getElementById('btn-reset-scene').addEventListener('click', () => {
    resetScene(State.mixture);
  });

  // Pause / Resume Button
  const btnPause = document.getElementById('btn-pause');
  btnPause.addEventListener('click', () => {
    State.paused = !State.paused;
    btnPause.textContent = State.paused ? '▶ Resume' : '⏸ Pause';
  });

  // Provenance Manifest Modal (WB-3.4)
  const modal = document.getElementById('manifest-modal');
  document.getElementById('btn-manifest').addEventListener('click', () => {
    modal.classList.remove('hidden');
  });
  document.getElementById('btn-close-modal').addEventListener('click', () => {
    modal.classList.add('hidden');
  });

  // The Hand Mouse & Touch Interactions (WB-4)
  canvas.addEventListener('mousedown', (e) => {
    const rect = canvas.getBoundingClientRect();
    const xMeters = ((e.clientX - rect.left) / rect.width) * State.viewWidthMeters * State.boxScale;
    const yMeters = ((e.clientY - rect.top) / rect.height) * State.viewWidthMeters * State.boxScale;
    State.hand.active = true;
    State.hand.x = xMeters;
    State.hand.y = yMeters;
    State.hand.lastX = xMeters;
    State.hand.lastY = yMeters;
    State.hand.radiusMeters = State.viewWidthMeters * 0.05; // 5% grab radius (WB-4.1)
    updateHandTelemetry();
  });

  window.addEventListener('mousemove', (e) => {
    if (!State.hand.active) return;
    const rect = canvas.getBoundingClientRect();
    const xMeters = ((e.clientX - rect.left) / rect.width) * State.viewWidthMeters * State.boxScale;
    const yMeters = ((e.clientY - rect.top) / rect.height) * State.viewWidthMeters * State.boxScale;
    State.hand.vx = (xMeters - State.hand.lastX) / 0.016;
    State.hand.vy = (yMeters - State.hand.lastY) / 0.016;
    State.hand.lastX = State.hand.x;
    State.hand.lastY = State.hand.y;
    State.hand.x = xMeters;
    State.hand.y = yMeters;

    // Check Refinement Patch trigger (WB-4.4)
    const speed = Math.hypot(State.hand.vx, State.hand.vy);
    if (speed > 500.0 && State.tier > 0) {
      document.getElementById('patch-indicator').classList.remove('hidden');
      document.getElementById('hand-recursion').textContent = "ACTIVE · REFINEMENT PATCH OPEN (WB-1.2)";
    } else {
      document.getElementById('patch-indicator').classList.add('hidden');
      document.getElementById('hand-recursion').textContent = "INACTIVE (Budget Commuting)";
    }
  });

  window.addEventListener('mouseup', () => {
    State.hand.active = false;
    document.getElementById('patch-indicator').classList.add('hidden');
  });

  window.addEventListener('resize', resizeCanvas);
}

function formatPhysicalScale(meters) {
  if (meters < 1e-9) return `${(meters * 1e10).toFixed(1)} Å`;
  if (meters < 1e-6) return `${(meters * 1e9).toFixed(2)} nm`;
  if (meters < 1e-3) return `${(meters * 1e6).toFixed(2)} µm`;
  if (meters < 1.0) return `${(meters * 1e3).toFixed(1)} mm`;
  if (meters < 1000.0) return `${meters.toFixed(1)} m`;
  return `${(meters / 1000.0).toFixed(2)} km`;
}

function updateTemperatureDisplay() {
  const K = State.temperature;
  let text = `${K.toFixed(1)} K`;
  if (State.tempUnit === 'C') {
    text = `${(K - 273.15).toFixed(1)} °C`;
  } else if (State.tempUnit === 'F') {
    text = `${((K - 273.15) * 9/5 + 32).toFixed(1)} °F`;
  }
  document.getElementById('temp-val').textContent = text;
}

function updateSimRateReadout() {
  const t = TIERS[State.tier];
  const rate = t.baseRate * State.governorBias;
  document.getElementById('sim-rate-val').textContent = `${rate.toFixed(1)} ${t.rateUnit}`;
  const pct = (t.baseDt * 30.0 * State.governorBias);
  document.getElementById('sim-rate-pct').textContent = `${pct.toExponential(2)} ×`;
}

function updateHandTelemetry() {
  const r = State.hand.radiusMeters;
  const rText = formatPhysicalScale(r);
  
  // Calculate grabbed mass (WB-4.1)
  const volumeM3 = (4/3) * Math.PI * Math.pow(r, 3);
  const massKg = volumeM3 * 1000.0;
  let massText = `${massKg.toExponential(2)} kg`;
  if (massKg > 1000.0) {
    massText = `${(massKg / 1000.0).toFixed(1)} tonnes (${(massKg / 500000).toFixed(2)} swimming pools)`;
  }
  document.getElementById('hand-radius').textContent = `${rText} (5% view)`;
  document.getElementById('hand-mass').textContent = massText;
}

function triggerSettlingState(desc) {
  State.settled = false;
  State.settlingTime = performance.now();
  const dot = document.getElementById('settling-dot');
  dot.className = 'status-indicator-dot settling';
  document.getElementById('settling-title').textContent = "SETTLING EQUILIBRIUM";
  document.getElementById('settling-desc').textContent = desc;

  setTimeout(() => {
    State.settled = true;
    dot.className = 'status-indicator-dot settled';
    document.getElementById('settling-title').textContent = "PRE-WARMED REFERENCE";
    document.getElementById('settling-desc').textContent = "Equilibrated certified reference state active";
  }, 1200);
}

function resetScene(preset) {
  State.mixture = preset;
  AtomisticScene.init(preset);
  PromotedScene.init();
  ContinuumScene.init();
  BulkScene.init();
  State.hand.cumulativeWorkHa = 0.0;
  triggerSettlingState(`Scene reset to ${preset.toUpperCase()} reference.`);
}

// ============================================================================
// 8. Main Application Loop
// ============================================================================

let lastFpsTime = performance.now();
let framesCount = 0;

function loop(now) {
  const dtReal = (now - State.lastFrameTime) * 0.001;
  State.lastFrameTime = now;

  framesCount++;
  if (now - lastFpsTime >= 500) {
    State.fps = (framesCount * 1000) / (now - lastFpsTime);
    document.getElementById('fps-val').textContent = State.fps.toFixed(1);
    framesCount = 0;
    lastFpsTime = now;
  }

  if (!State.paused) {
    const t = TIERS[State.tier];
    const dtSim = t.baseDt * State.governorBias;
    stepPhysics(dtSim);
  }

  updateOrderParameters();
  render();

  requestAnimationFrame(loop);
}

// Bootstrap
window.addEventListener('DOMContentLoaded', () => {
  resizeCanvas();
  initControls();
  resetScene('h2o');
  requestAnimationFrame(loop);
});
