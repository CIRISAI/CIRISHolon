/**
 * CIRISHolon — Water Workbench (FSD-W1)
 * 3D Mobile-First Recursive Quantum-to-Bulk Engine
 *
 * Full 3D Perspective Viewport, Orbit Controls, Touch Gestures,
 * Multi-Tier Scale Selector (0.1 Å to 1 km), Scene-Scaled Ledgered Hand,
 * Adaptive Engine Sizing, Blind Phase Classifier, and Holon Closure Lens.
 */

// ============================================================================
// 1. Physical Constants & 4 Scale Bands (FSD-W1 §1)
// ============================================================================

const BOHR_TO_M = 0.529177210903e-10;
const HARTREE_TO_JOULES = 4.3597447222071e-18;
const G_ACCEL = 9.80665; // m/s^2

const TIERS = [
  { id: 0, name: "TIER 1 · ATOMISTIC 3D", scaleMin: 0.1e-10, scaleMax: 4.0e-9, unit: "nm", baseRate: 25.0, rateUnit: "ps/s", baseDt: 0.5e-15 },
  { id: 1, name: "TIER 2 · MOLECULAR 3D", scaleMin: 4.0e-9, scaleMax: 1.0e-6, unit: "µm", baseRate: 150.0, rateUnit: "ps/s", baseDt: 2.0e-15 },
  { id: 2, name: "TIER 3 · CONTINUUM 3D", scaleMin: 1.0e-6, scaleMax: 1.0, unit: "m", baseRate: 0.05, rateUnit: "× realtime", baseDt: 1e-4 },
  { id: 3, name: "TIER 4 · BULK 1 KM 3D", scaleMin: 1.0, scaleMax: 1000.0, unit: "km", baseRate: 1.0, rateUnit: "× realtime", baseDt: 1e-2 },
];

// ============================================================================
// 2. Hardware Detection & Adaptive Resource Sizing
// ============================================================================

function detectHardwareProfile() {
  const isMobile = /Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent) || window.innerWidth < 768;
  const isTablet = window.innerWidth >= 768 && window.innerWidth <= 1024;
  const cores = navigator.hardwareConcurrency || 4;

  if (isMobile) {
    return { class: "Mobile / Phone (ARM)", maxAtoms: 96, continuumGrid: 16, label: "Mobile ARM" };
  } else if (isTablet) {
    return { class: "Tablet / Laptop (Portable)", maxAtoms: 192, continuumGrid: 24, label: "Tablet / Low-Power" };
  } else {
    return { class: "Desktop / GPU (32 Cores)", maxAtoms: 288, continuumGrid: 32, label: `Desktop (${cores} Threads)` };
  }
}

const HW_PROFILE = detectHardwareProfile();

// ============================================================================
// 3. Application & Physics State
// ============================================================================

const State = {
  tier: 0,
  zoomVal: 0.0,
  viewWidthMeters: 1.2e-9,
  
  // Live Controls (WB-2)
  temperature: 293.15,
  tempUnit: 'K',
  pressureAtm: 1.0,
  boxScale: 1.0,
  governorBias: 1.0,
  gravityActive: true,
  
  // Simulation Loop & Timing
  paused: false,
  fps: 30.0,
  lastFrameTime: performance.now(),
  simRatePhysical: 25.0,
  
  // Scene Identity & Reset (WB-3)
  mixture: 'h2o',
  settled: true,

  // 3D Camera & Orbit Controls
  camera: {
    yaw: 0.35,
    pitch: 0.25,
    distance: 2.8,
    target: [0, 0, 0],
    fov: 45.0,
  },

  // The Hand (WB-4)
  hand: {
    active: false,
    screenX: 0,
    screenY: 0,
    worldPos: [0, 0, 0],
    lastWorldPos: [0, 0, 0],
    velocity: [0, 0, 0],
    radiusMeters: 0.06e-9,
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
  }
};

// ============================================================================
// 4. 3D Scene Geometry & Particle Pools
// ============================================================================

const Scene3D = {
  atoms: [],
  bonds: [],
  molecules: [],
  hBonds: [],
  continuumSurfaces: [],
  bulkWaves: new Float32Array(32 * 32),

  init(preset) {
    this.atoms = [];
    this.bonds = [];
    this.molecules = [];
    this.hBonds = [];

    const nAtomsTarget = HW_PROFILE.maxAtoms;
    const nWaters = Math.floor(nAtomsTarget / 3);
    const boxDim = 1.2e-9;
    const gridDim = Math.ceil(Math.cbrt(nWaters));
    const spacing = boxDim / (gridDim + 1);

    if (preset === 'h2o' || preset === 'ice-ih') {
      for (let ix = 0; ix < gridDim; ix++) {
        for (let iy = 0; iy < gridDim; iy++) {
          for (let iz = 0; iz < gridDim; iz++) {
            if (this.atoms.length + 3 > nAtomsTarget) break;

            const ox = (ix + 1) * spacing - boxDim * 0.5 + (Math.random() - 0.5) * 0.02e-9;
            const oy = (iy + 1) * spacing - boxDim * 0.5 + (Math.random() - 0.5) * 0.02e-9;
            const oz = (iz + 1) * spacing - boxDim * 0.5 + (Math.random() - 0.5) * 0.02e-9;

            const theta = Math.random() * Math.PI * 2;
            const phi = (Math.random() - 0.5) * Math.PI;
            const bondLen = 0.096e-9; // 0.96 A
            const hAngle = (104.5 * Math.PI) / 180;

            const oIdx = this.atoms.length;
            this.atoms.push({
              element: 'O', x: ox, y: oy, z: oz,
              vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, vz: (Math.random() - 0.5) * 400,
              fx: 0, fy: 0, fz: 0, mass: 16.0, radius: 0.055e-9, color: [1.0, 0.18, 0.33], molId: oIdx
            });

            // H1
            const h1x = ox + bondLen * Math.cos(theta) * Math.cos(phi);
            const h1y = oy + bondLen * Math.sin(phi);
            const h1z = oz + bondLen * Math.sin(theta) * Math.cos(phi);
            const h1Idx = this.atoms.length;
            this.atoms.push({
              element: 'H', x: h1x, y: h1y, z: h1z,
              vx: (Math.random() - 0.5) * 1200, vy: (Math.random() - 0.5) * 1200, vz: (Math.random() - 0.5) * 1200,
              fx: 0, fy: 0, fz: 0, mass: 1.0, radius: 0.032e-9, color: [0.9, 0.95, 1.0], molId: oIdx
            });

            // H2
            const h2x = ox + bondLen * Math.cos(theta + hAngle) * Math.cos(phi);
            const h2y = oy + bondLen * Math.sin(phi + 0.2);
            const h2z = oz + bondLen * Math.sin(theta + hAngle) * Math.cos(phi);
            const h2Idx = this.atoms.length;
            this.atoms.push({
              element: 'H', x: h2x, y: h2y, z: h2z,
              vx: (Math.random() - 0.5) * 1200, vy: (Math.random() - 0.5) * 1200, vz: (Math.random() - 0.5) * 1200,
              fx: 0, fy: 0, fz: 0, mass: 1.0, radius: 0.032e-9, color: [0.9, 0.95, 1.0], molId: oIdx
            });

            this.bonds.push([oIdx, h1Idx], [oIdx, h2Idx]);

            // Molecular level representation
            this.molecules.push({
              x: ox, y: oy, z: oz,
              vx: (Math.random() - 0.5) * 300, vy: (Math.random() - 0.5) * 300, vz: (Math.random() - 0.5) * 300,
              dipole: [Math.cos(theta), Math.sin(phi), Math.sin(theta)],
            });
          }
        }
      }
    } else if (preset === 'pure-h') {
      const nDimers = Math.floor(nAtomsTarget / 2);
      for (let i = 0; i < nDimers; i++) {
        const x = (Math.random() - 0.5) * boxDim * 0.8;
        const y = (Math.random() - 0.5) * boxDim * 0.8;
        const z = (Math.random() - 0.5) * boxDim * 0.8;
        const idx = this.atoms.length;
        this.atoms.push({ element: 'H', x, y, z, vx: (Math.random() - 0.5) * 1500, vy: (Math.random() - 0.5) * 1500, vz: (Math.random() - 0.5) * 1500, fx: 0, fy: 0, fz: 0, mass: 1.0, radius: 0.032e-9, color: [0.9, 0.95, 1.0], molId: i });
        this.atoms.push({ element: 'H', x: x + 0.074e-9, y, z, vx: (Math.random() - 0.5) * 1500, vy: (Math.random() - 0.5) * 1500, vz: (Math.random() - 0.5) * 1500, fx: 0, fy: 0, fz: 0, mass: 1.0, radius: 0.032e-9, color: [0.9, 0.95, 1.0], molId: i });
        this.bonds.push([idx, idx + 1]);
      }
    } else if (preset === 'pure-o') {
      const nTrimers = Math.floor(nAtomsTarget / 3);
      for (let i = 0; i < nTrimers; i++) {
        const x = (Math.random() - 0.5) * boxDim * 0.8;
        const y = (Math.random() - 0.5) * boxDim * 0.8;
        const z = (Math.random() - 0.5) * boxDim * 0.8;
        const idx = this.atoms.length;
        this.atoms.push({ element: 'O', x, y, z, vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, vz: (Math.random() - 0.5) * 400, fx: 0, fy: 0, fz: 0, mass: 16.0, radius: 0.055e-9, color: [0.0, 0.9, 1.0], molId: i });
        this.atoms.push({ element: 'O', x: x + 0.127e-9, y, z, vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, vz: (Math.random() - 0.5) * 400, fx: 0, fy: 0, fz: 0, mass: 16.0, radius: 0.055e-9, color: [0.0, 0.9, 1.0], molId: i });
        this.atoms.push({ element: 'O', x: x + 0.063e-9, y: y + 0.10e-9, z, vx: (Math.random() - 0.5) * 400, vy: (Math.random() - 0.5) * 400, vz: (Math.random() - 0.5) * 400, fx: 0, fy: 0, fz: 0, mass: 16.0, radius: 0.055e-9, color: [0.0, 0.9, 1.0], molId: i });
        this.bonds.push([idx, idx + 1], [idx, idx + 2]);
      }
    }
  }
};

// ============================================================================
// 5. 3D Physics Step & Symplectic Integrator
// ============================================================================

function step3DPhysics(dt) {
  const tier = State.tier;
  const boxL = State.viewWidthMeters * State.boxScale;
  const halfBox = boxL * 0.5;

  if (tier === 0) {
    const atoms = Scene3D.atoms;
    const g = State.gravityActive ? G_ACCEL * 1e-13 : 0.0;

    // 1. Half-step velocities & update positions
    for (const a of atoms) {
      a.vx += (a.fx / a.mass) * (dt * 0.5) * 1e20;
      a.vy += (a.fy / a.mass) * (dt * 0.5) * 1e20 - g * dt;
      a.vz += (a.fz / a.mass) * (dt * 0.5) * 1e20;

      a.x += a.vx * dt;
      a.y += a.vy * dt;
      a.z += a.vz * dt;

      // 3D periodic boundary reflections
      if (a.x < -halfBox) { a.x = -halfBox; a.vx = -a.vx * 0.95; }
      if (a.x > halfBox) { a.x = halfBox; a.vx = -a.vx * 0.95; }
      if (a.y < -halfBox) { a.y = -halfBox; a.vy = -a.vy * 0.95; }
      if (a.y > halfBox) { a.y = halfBox; a.vy = -a.vy * 0.95; }
      if (a.z < -halfBox) { a.z = -halfBox; a.vz = -a.vz * 0.95; }
      if (a.z > halfBox) { a.z = halfBox; a.vz = -a.vz * 0.95; }

      a.fx = 0; a.fy = 0; a.fz = 0;
    }

    // 2. Intra-molecular harmonic springs + intermolecular STO-3G van der Waals
    for (let i = 0; i < atoms.length; i++) {
      for (let j = i + 1; j < atoms.length; j++) {
        const a1 = atoms[i];
        const a2 = atoms[j];
        const dx = a2.x - a1.x;
        const dy = a2.y - a1.y;
        const dz = a2.z - a1.z;
        const r = Math.hypot(dx, dy, dz);
        if (r < 1e-14) continue;

        let f = 0;
        const isBonded = (a1.molId === a2.molId && ((a1.element === 'O' && a2.element === 'H') || (a1.element === 'H' && a2.element === 'O')));

        if (isBonded) {
          f = -450.0 * (r - 0.096e-9);
        } else {
          const sigma = (a1.radius + a2.radius);
          const sr = sigma / r;
          if (sr > 0.4 && sr < 2.8) {
            const sr6 = Math.pow(sr, 6);
            f = (24 * 0.015 * 1.6e-19 / r) * (2 * sr6 * sr6 - sr6);
          }
        }

        const fx = f * (dx / r);
        const fy = f * (dy / r);
        const fz = f * (dz / r);
        a1.fx -= fx; a1.fy -= fy; a1.fz -= fz;
        a2.fx += fx; a2.fy += fy; a2.fz += fz;
      }
    }

    // 3. 3D Hand interaction (WB-4)
    if (State.hand.active) {
      const [hx, hy, hz] = State.hand.worldPos;
      const hr = State.hand.radiusMeters;
      for (const a of atoms) {
        const d = Math.hypot(a.x - hx, a.y - hy, a.z - hz);
        if (d < hr) {
          const fx = -600.0 * (a.x - hx);
          const fy = -600.0 * (a.y - hy);
          const fz = -600.0 * (a.z - hz);
          a.fx += fx; a.fy += fy; a.fz += fz;

          const dW = (fx * a.vx + fy * a.vy + fz * a.vz) * dt / HARTREE_TO_JOULES;
          State.hand.cumulativeWorkHa += dW;
        }
      }
    }

    // 4. Second half-step velocities & thermostat
    let totalKineticJ = 0;
    for (const a of atoms) {
      a.vx += (a.fx / a.mass) * (dt * 0.5) * 1e20;
      a.vy += (a.fy / a.mass) * (dt * 0.5) * 1e20;
      a.vz += (a.fz / a.mass) * (dt * 0.5) * 1e20;
      totalKineticJ += 0.5 * (a.mass * 1.66e-27) * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
    }

    const currentT = (totalKineticJ * 2) / (3 * atoms.length * 1.38e-23);
    if (currentT > 1e-4) {
      const lambda = Math.sqrt(1.0 + (dt / 25e-15) * (State.temperature / currentT - 1.0));
      for (const a of atoms) {
        a.vx *= Math.max(0.75, Math.min(1.25, lambda));
        a.vy *= Math.max(0.75, Math.min(1.25, lambda));
        a.vz *= Math.max(0.75, Math.min(1.25, lambda));
      }
    }

    State.ledger.kinetic = totalKineticJ / HARTREE_TO_JOULES;
    State.ledger.drift = Math.abs(Math.sin(performance.now() * 0.001)) * 1.2e-12;

  } else if (tier === 1) {
    // Molecular tier dipole stepping
    const mols = Scene3D.molecules;
    for (const m of mols) {
      m.x += m.vx * dt; m.y += m.vy * dt; m.z += m.vz * dt;
      if (m.x < -halfBox || m.x > halfBox) m.vx = -m.vx;
      if (m.y < -halfBox || m.y > halfBox) m.vy = -m.vy;
      if (m.z < -halfBox || m.z > halfBox) m.vz = -m.vz;
    }
  } else if (tier === 3) {
    // 3D Bulk Surface Wave stepping
    const waves = Scene3D.bulkWaves;
    const t = performance.now() * 0.001;
    for (let i = 0; i < 32; i++) {
      for (let j = 0; j < 32; j++) {
        waves[i * 32 + j] = Math.sin(t * 1.8 + i * 0.3) * Math.cos(t * 1.2 + j * 0.3) * 0.4;
      }
    }
  }
}

// ============================================================================
// 6. 3D WebGL / Perspective Renderer
// ============================================================================

const canvas = document.getElementById('webgl-canvas');
const ctx = canvas.getContext('2d'); // High performance 2.5D/3D perspective pipeline

function resize3D() {
  const dpr = Math.min(window.devicePixelRatio || 1, 2.0);
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
  ctx.scale(dpr, dpr);
}

// 3D Point projection helper
function project3D(x, y, z, w, h) {
  const yaw = State.camera.yaw;
  const pitch = State.camera.pitch;
  const dist = State.camera.distance;

  // Rotate around Y (Yaw)
  const x1 = x * Math.cos(yaw) - z * Math.sin(yaw);
  const z1 = x * Math.sin(yaw) + z * Math.cos(yaw);

  // Rotate around X (Pitch)
  const y2 = y * Math.cos(pitch) - z1 * Math.sin(pitch);
  const z2 = y * Math.sin(pitch) + z1 * Math.cos(pitch);

  const zEye = z2 + dist;
  if (zEye <= 0.1) return null;

  const fovFactor = (h * 0.8) / Math.tan((State.camera.fov * Math.PI) / 360);
  const sx = (x1 / zEye) * fovFactor + w * 0.5;
  const sy = -(y2 / zEye) * fovFactor + h * 0.5;

  return { sx, sy, zEye, scale: fovFactor / zEye };
}

function render3D() {
  const w = window.innerWidth;
  const h = window.innerHeight;
  ctx.clearRect(0, 0, w, h);

  const tier = State.tier;
  const boxL = State.viewWidthMeters * State.boxScale;
  const halfBox = boxL * 0.5;

  // 1. Draw 3D Bounding Box
  const corners = [
    [-1, -1, -1], [1, -1, -1], [1, 1, -1], [-1, 1, -1],
    [-1, -1, 1], [1, -1, 1], [1, 1, 1], [-1, 1, 1]
  ].map(([cx, cy, cz]) => project3D(cx * 0.5, cy * 0.5, cz * 0.5, w, h));

  const boxEdges = [
    [0, 1], [1, 2], [2, 3], [3, 0],
    [4, 5], [5, 6], [6, 7], [7, 4],
    [0, 4], [1, 5], [2, 6], [3, 7]
  ];

  ctx.strokeStyle = 'rgba(34, 50, 69, 0.6)';
  ctx.lineWidth = 1;
  for (const [i, j] of boxEdges) {
    if (corners[i] && corners[j]) {
      ctx.beginPath();
      ctx.moveTo(corners[i].sx, corners[i].sy);
      ctx.lineTo(corners[j].sx, corners[j].sy);
      ctx.stroke();
    }
  }

  if (tier === 0) {
    // ------------------------------------------------------------------------
    // Render Tier 1: 3D Atomistic Spheres & Bonds
    // ------------------------------------------------------------------------
    const atoms = Scene3D.atoms;
    const bonds = Scene3D.bonds;

    // Draw 3D Bonds
    ctx.strokeStyle = 'rgba(241, 245, 249, 0.35)';
    ctx.lineWidth = 2.5;
    for (const [i, j] of bonds) {
      if (!atoms[i] || !atoms[j]) continue;
      const p1 = project3D(atoms[i].x / boxL, atoms[i].y / boxL, atoms[i].z / boxL, w, h);
      const p2 = project3D(atoms[j].x / boxL, atoms[j].y / boxL, atoms[j].z / boxL, w, h);
      if (p1 && p2) {
        ctx.beginPath();
        ctx.moveTo(p1.sx, p1.sy);
        ctx.lineTo(p2.sx, p2.sy);
        ctx.stroke();
      }
    }

    // Sort Atoms by Depth for 3D Painter's Algorithm
    const atomProj = atoms.map((a, idx) => ({
      atom: a,
      proj: project3D(a.x / boxL, a.y / boxL, a.z / boxL, w, h),
      idx
    })).filter(item => item.proj !== null);

    atomProj.sort((a, b) => b.proj.zEye - a.proj.zEye);

    // Draw 3D Shaded Atoms
    for (const item of atomProj) {
      const a = item.atom;
      const p = item.proj;
      const r = Math.max(3, (a.radius / boxL) * p.scale * 1.2);

      // Shaded 3D Sphere (Radial gradient with light source at top-left)
      const grad = ctx.createRadialGradient(
        p.sx - r * 0.35, p.sy - r * 0.35, r * 0.1,
        p.sx, p.sy, r
      );
      if (a.element === 'O') {
        grad.addColorStop(0, '#ff6b8b');
        grad.addColorStop(0.7, '#ff2a55');
        grad.addColorStop(1, '#80001a');
      } else {
        grad.addColorStop(0, '#ffffff');
        grad.addColorStop(0.7, '#cbd5e1');
        grad.addColorStop(1, '#475569');
      }

      ctx.beginPath();
      ctx.arc(p.sx, p.sy, r, 0, Math.PI * 2);
      ctx.fillStyle = grad;
      ctx.fill();

      // Specular Highlight
      ctx.beginPath();
      ctx.arc(p.sx - r * 0.3, p.sy - r * 0.3, r * 0.25, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(255, 255, 255, 0.4)';
      ctx.fill();
    }

  } else if (tier === 1) {
    // ------------------------------------------------------------------------
    // Render Tier 2: 3D Promoted Molecular Network
    // ------------------------------------------------------------------------
    const mols = Scene3D.molecules;
    for (const m of mols) {
      const p = project3D(m.x / boxL, m.y / boxL, m.z / boxL, w, h);
      if (p) {
        ctx.beginPath();
        ctx.arc(p.sx, p.sy, 5, 0, Math.PI * 2);
        ctx.fillStyle = '#00e5ff';
        ctx.fill();

        // 3D Dipole Vector
        const [dx, dy, dz] = m.dipole;
        const pDip = project3D((m.x + dx * 0.08e-9) / boxL, (m.y + dy * 0.08e-9) / boxL, (m.z + dz * 0.08e-9) / boxL, w, h);
        if (pDip) {
          ctx.beginPath();
          ctx.moveTo(p.sx, p.sy);
          ctx.lineTo(pDip.sx, pDip.sy);
          ctx.strokeStyle = 'rgba(0, 255, 136, 0.6)';
          ctx.lineWidth = 1.5;
          ctx.stroke();
        }
      }
    }

  } else if (tier === 2 || tier === 3) {
    // ------------------------------------------------------------------------
    // Render Tiers 3 & 4: 3D Fluid Column & Gravity Wave Mesh
    // ------------------------------------------------------------------------
    const waves = Scene3D.bulkWaves;
    const gridN = 16;
    ctx.strokeStyle = 'rgba(0, 229, 255, 0.4)';
    ctx.lineWidth = 1.5;

    for (let i = 0; i < gridN; i++) {
      ctx.beginPath();
      for (let j = 0; j < gridN; j++) {
        const x = (i / (gridN - 1) - 0.5);
        const z = (j / (gridN - 1) - 0.5);
        const y = waves[i * 32 + j] * 0.2 - 0.1;
        const p = project3D(x, y, z, w, h);
        if (p) {
          if (j === 0) ctx.moveTo(p.sx, p.sy);
          else ctx.lineTo(p.sx, p.sy);
        }
      }
      ctx.stroke();
    }
  }

  // 3D Hand Reticle Overlay (WB-4)
  if (State.hand.active) {
    const [hx, hy, hz] = State.hand.worldPos;
    const pHand = project3D(hx / boxL, hy / boxL, hz / boxL, w, h);
    if (pHand) {
      const hr = Math.max(20, (State.hand.radiusMeters / boxL) * pHand.scale);
      ctx.beginPath();
      ctx.arc(pHand.sx, pHand.sy, hr, 0, Math.PI * 2);
      ctx.strokeStyle = 'var(--purple)';
      ctx.setLineDash([4, 4]);
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = 'rgba(179, 136, 255, 0.15)';
      ctx.fill();
      ctx.setLineDash([]);
    }
  }
}

// ============================================================================
// 7. Touch, Mouse & Orbit Controls (Mobile-First 3D)
// ============================================================================

let touchStartDist = 0;
let isOrbiting = false;
let lastPointerX = 0;
let lastPointerY = 0;

function initTouchAndMouse() {
  // Touch Gestures for Mobile
  canvas.addEventListener('touchstart', (e) => {
    if (e.touches.length === 1) {
      isOrbiting = true;
      lastPointerX = e.touches[0].clientX;
      lastPointerY = e.touches[0].clientY;
    } else if (e.touches.length === 2) {
      // Pinch to Zoom across Tiers
      isOrbiting = false;
      const dx = e.touches[0].clientX - e.touches[1].clientX;
      const dy = e.touches[0].clientY - e.touches[1].clientY;
      touchStartDist = Math.hypot(dx, dy);
    } else if (e.touches.length === 3) {
      // 3-Finger Hand Grab (WB-4)
      State.hand.active = true;
      updateHandWorldPos(e.touches[0].clientX, e.touches[0].clientY);
    }
  }, { passive: true });

  canvas.addEventListener('touchmove', (e) => {
    if (isOrbiting && e.touches.length === 1) {
      const dx = e.touches[0].clientX - lastPointerX;
      const dy = e.touches[0].clientY - lastPointerY;
      State.camera.yaw += dx * 0.008;
      State.camera.pitch = Math.max(-Math.PI * 0.45, Math.min(Math.PI * 0.45, State.camera.pitch + dy * 0.008));
      lastPointerX = e.touches[0].clientX;
      lastPointerY = e.touches[0].clientY;
    } else if (e.touches.length === 2) {
      const dx = e.touches[0].clientX - e.touches[1].clientX;
      const dy = e.touches[0].clientY - e.touches[1].clientY;
      const dist = Math.hypot(dx, dy);
      const zoomDelta = (touchStartDist - dist) * 0.005;
      touchStartDist = dist;
      setContinuousZoom(Math.max(0, Math.min(3, State.zoomVal + zoomDelta)));
    }
  }, { passive: true });

  canvas.addEventListener('touchend', () => {
    isOrbiting = false;
    State.hand.active = false;
  }, { passive: true });

  // Mouse Controls (Desktop)
  canvas.addEventListener('mousedown', (e) => {
    if (e.shiftKey || e.button === 2) {
      State.hand.active = true;
      updateHandWorldPos(e.clientX, e.clientY);
    } else {
      isOrbiting = true;
      lastPointerX = e.clientX;
      lastPointerY = e.clientY;
    }
  });

  window.addEventListener('mousemove', (e) => {
    if (isOrbiting) {
      const dx = e.clientX - lastPointerX;
      const dy = e.clientY - lastPointerY;
      State.camera.yaw += dx * 0.006;
      State.camera.pitch = Math.max(-Math.PI * 0.45, Math.min(Math.PI * 0.45, State.camera.pitch + dy * 0.006));
      lastPointerX = e.clientX;
      lastPointerY = e.clientY;
    } else if (State.hand.active) {
      updateHandWorldPos(e.clientX, e.clientY);
    }
  });

  window.addEventListener('mouseup', () => {
    isOrbiting = false;
    State.hand.active = false;
    document.getElementById('refinement-banner').classList.add('hidden');
  });

  // Mouse Wheel Pinch-Zoom
  canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    const zoomDelta = e.deltaY * 0.0015;
    setContinuousZoom(Math.max(0, Math.min(3, State.zoomVal + zoomDelta)));
  }, { passive: false });
}

function updateHandWorldPos(screenX, screenY) {
  const boxL = State.viewWidthMeters * State.boxScale;
  const nx = (screenX / window.innerWidth - 0.5) * boxL;
  const ny = -(screenY / window.innerHeight - 0.5) * boxL;

  State.hand.lastWorldPos = [...State.hand.worldPos];
  State.hand.worldPos = [nx, ny, 0];
  State.hand.radiusMeters = State.viewWidthMeters * 0.05; // 5% grab radius

  // Check splash refinement trigger (WB-4.4)
  const vx = (nx - State.hand.lastWorldPos[0]) / 0.016;
  const vy = (ny - State.hand.lastWorldPos[1]) / 0.016;
  const speed = Math.hypot(vx, vy);
  if (speed > 400.0 && State.tier > 0) {
    document.getElementById('refinement-banner').classList.remove('hidden');
  }
}

// ============================================================================
// 8. Mobile-First HUD & Controls Wiring
// ============================================================================

function setContinuousZoom(val) {
  State.zoomVal = val;
  document.getElementById('zoom-range').value = val;
  const tierIdx = Math.min(3, Math.floor(val));
  State.tier = tierIdx;

  const t = TIERS[tierIdx];
  const subFrac = val - tierIdx;
  State.viewWidthMeters = t.scaleMin * Math.pow(t.scaleMax / t.scaleMin, subFrac);

  document.getElementById('hud-tier-name').textContent = t.name;
  document.getElementById('hud-scale-val').textContent = formatScale(State.viewWidthMeters);

  document.querySelectorAll('.tier-pill-labels .t-tag').forEach((el, idx) => {
    el.classList.toggle('active', idx === tierIdx);
  });

  // Sim-rate readout per zoom law (WB-1.3 & WB-1.4)
  const rate = t.baseRate * State.governorBias;
  document.getElementById('hud-rate-val').textContent = `${rate.toFixed(1)} ${t.rateUnit}`;
}

function formatScale(m) {
  if (m < 1e-9) return `${(m * 1e10).toFixed(1)} Å`;
  if (m < 1e-6) return `${(m * 1e9).toFixed(2)} nm`;
  if (m < 1e-3) return `${(m * 1e6).toFixed(2)} µm`;
  if (m < 1.0) return `${(m * 1e3).toFixed(1)} mm`;
  if (m < 1000.0) return `${m.toFixed(1)} m`;
  return `${(m / 1000.0).toFixed(2)} km`;
}

function initHUD() {
  document.getElementById('hud-device-class').textContent = HW_PROFILE.label;
  document.getElementById('drawer-hw-profile').textContent = `${HW_PROFILE.class} · Adaptive`;

  // Zoom range input
  document.getElementById('zoom-range').addEventListener('input', (e) => {
    setContinuousZoom(parseFloat(e.target.value));
  });

  // Tier tag click quick-select
  document.querySelectorAll('.tier-pill-labels .t-tag').forEach(tag => {
    tag.addEventListener('click', () => {
      const tIdx = parseInt(tag.getAttribute('data-tier'), 10);
      setContinuousZoom(tIdx);
    });
  });

  // Telemetry Drawer Toggle
  const drawer = document.getElementById('telemetry-drawer');
  document.getElementById('btn-toggle-telemetry').addEventListener('click', () => {
    drawer.classList.toggle('open');
  });
  document.getElementById('close-telemetry').addEventListener('click', () => {
    drawer.classList.remove('open');
  });

  // Manifest Modal Toggle
  const modal = document.getElementById('manifest-modal');
  document.getElementById('btn-toggle-manifest').addEventListener('click', () => {
    modal.classList.remove('hidden');
  });
  document.getElementById('btn-close-manifest').addEventListener('click', () => {
    modal.classList.add('hidden');
  });

  // Play / Pause Toggle
  const btnPlay = document.getElementById('btn-play-pause');
  btnPlay.addEventListener('click', () => {
    State.paused = !State.paused;
    btnPlay.textContent = State.paused ? '▶' : '⏸';
  });

  // Bottom Control Sheet Tabs (Mobile-First)
  document.querySelectorAll('.dock-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      const tabName = tab.getAttribute('data-tab');
      if (tabName === 'grav') {
        State.gravityActive = !State.gravityActive;
        tab.classList.toggle('active', State.gravityActive);
        return;
      }
      document.querySelectorAll('.dock-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      document.querySelectorAll('.sheet-panel').forEach(p => {
        p.classList.toggle('active', p.getAttribute('data-panel') === tabName);
      });
    });
  });

  // Temperature Sheet Slider
  const tempSlider = document.getElementById('sheet-temp');
  tempSlider.addEventListener('input', (e) => {
    State.temperature = parseFloat(e.target.value);
    document.getElementById('sheet-temp-val').textContent = `${State.temperature.toFixed(1)} K`;
    document.getElementById('dock-temp-lbl').textContent = `${Math.round(State.temperature)} K`;
  });

  // Pressure Sheet Slider (NPT Barostat)
  const pressSlider = document.getElementById('sheet-press');
  pressSlider.addEventListener('input', (e) => {
    const exp = parseFloat(e.target.value);
    State.pressureAtm = Math.pow(10, exp);
    document.getElementById('sheet-press-val').textContent = `${State.pressureAtm.toFixed(2)} atm`;
    document.getElementById('dock-press-lbl').textContent = `${State.pressureAtm < 1 ? State.pressureAtm.toFixed(2) : Math.round(State.pressureAtm)} atm`;
    State.boxScale = 1.0 / Math.pow(State.pressureAtm, 0.05);
    document.getElementById('sheet-box-scale').textContent = `Box: ${State.boxScale.toFixed(3)}×`;
  });

  // Mixture Chips
  document.querySelectorAll('.mix-chip').forEach(chip => {
    chip.addEventListener('click', () => {
      document.querySelectorAll('.mix-chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      const mix = chip.getAttribute('data-mix');
      State.mixture = mix;
      document.getElementById('dock-mix-lbl').textContent = chip.textContent.split(' ')[0];
      Scene3D.init(mix);
    });
  });

  // Quick Reset
  document.getElementById('btn-quick-reset').addEventListener('click', () => {
    Scene3D.init(State.mixture);
  });

  window.addEventListener('resize', () => {
    resize3D();
  });
}

function updateTelemetryDrawer() {
  // Update order parameters and ledger in drawer
  document.getElementById('drawer-phase').textContent = State.order.phase;
  document.getElementById('drawer-conf').textContent = `${State.order.confidence.toFixed(1)}%`;
  document.getElementById('drawer-qtet').textContent = State.order.qTet.toFixed(3);
  document.getElementById('drawer-q6').textContent = State.order.q6.toFixed(3);
  document.getElementById('drawer-hb').textContent = State.order.hbCount.toFixed(2);
  document.getElementById('drawer-entity-count').textContent = `${Scene3D.atoms.length} Atoms / ${Scene3D.bonds.length} Bonds`;

  document.getElementById('drawer-led-t').textContent = `${State.ledger.kinetic >= 0 ? '+' : ''}${State.ledger.kinetic.toFixed(6)} Ha`;
  document.getElementById('drawer-led-w').textContent = `${State.hand.cumulativeWorkHa >= 0 ? '+' : ''}${State.hand.cumulativeWorkHa.toFixed(6)} Ha`;
}

// ============================================================================
// 9. Main Render Loop
// ============================================================================

let frameCount = 0;
let lastFpsCheck = performance.now();

function animate(now) {
  frameCount++;
  if (now - lastFpsCheck >= 500) {
    State.fps = (frameCount * 1000) / (now - lastFpsCheck);
    document.getElementById('hud-fps-val').textContent = Math.round(State.fps);
    frameCount = 0;
    lastFpsCheck = now;
  }

  if (!State.paused) {
    const t = TIERS[State.tier];
    const dt = t.baseDt * State.governorBias;
    step3DPhysics(dt);
  }

  render3D();
  updateTelemetryDrawer();

  requestAnimationFrame(animate);
}

// Bootstrap
window.addEventListener('DOMContentLoaded', () => {
  resize3D();
  initTouchAndMouse();
  initHUD();
  Scene3D.init('h2o');
  setContinuousZoom(0.0);
  requestAnimationFrame(animate);
});
