// MilestoneTracker.tsx
// TypeScript component for Univrs ecosystem milestone visualization
// Add to: ~/repos/univrs.io/src/components/ or ~/repos/vudo.univrs.io/src/components/

import { useState, useEffect } from 'react';

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

interface Phase {
  id: string;
  name: string;
  status: 'complete' | 'active' | 'pending';
  tests: number;
  description: string;
  deliverables: string[];
}

interface Repository {
  name: string;
  url: string;
  status: 'stable' | 'active' | 'pending';
  tests: number;
}

interface ENRSubsystem {
  name: string;
  status: 'complete' | 'active' | 'pending';
  dolLines: number;
  formula?: string;
}

// ═══════════════════════════════════════════════════════════════════════════
// DATA
// ═══════════════════════════════════════════════════════════════════════════

const phases: Phase[] = [
  {
    id: 'phase1',
    name: 'Phase 1: Parser + Lexer',
    status: 'complete',
    tests: 150,
    description: 'DOL language parsing foundation',
    deliverables: ['Lexer', 'Parser', 'AST', 'Error recovery'],
  },
  {
    id: 'phase2a',
    name: 'Phase 2a: HIR v0.4.0',
    status: 'complete',
    tests: 466,
    description: 'High-level Intermediate Representation',
    deliverables: ['HirModule', 'HirDecl', 'HirExpr', 'Type system'],
  },
  {
    id: 'phase2b',
    name: 'Phase 2b: VUDO VM',
    status: 'complete',
    tests: 402,
    description: 'WebAssembly virtual machine',
    deliverables: ['Wasmtime runtime', 'Sandbox', 'Fuel metering', 'Host functions'],
  },
  {
    id: 'phase2c',
    name: 'Phase 2c: Spirit Runtime',
    status: 'complete',
    tests: 50,
    description: 'Capability-based agent system',
    deliverables: ['Spirit registry', 'Manifest', 'Capabilities', 'Lifecycle'],
  },
  {
    id: 'phase3',
    name: 'Phase 3: MLIR + WASM Pipeline',
    status: 'complete',
    tests: 50,
    description: 'DOL → HIR → MLIR → WASM compilation',
    deliverables: ['MLIR lowering', 'WASM backend', 'add.wasm validated'],
  },
  {
    id: 'phase4a',
    name: 'Phase 4a: Hyphal Network',
    status: 'complete',
    tests: 38,
    description: 'Biology-inspired distributed patterns',
    deliverables: ['Topology', 'Discovery', 'Growth', 'Swarm coordinator'],
  },
  {
    id: 'phase4b',
    name: 'Phase 4b: ENR Economic Layer',
    status: 'active',
    tests: 0,
    description: 'Entropy-Nexus-Revival primitives',
    deliverables: ['Core types', 'Entropy calculator', 'Nexus topology', 'Revival pool'],
  },
];

const repositories: Repository[] = [
  { name: 'univrs-dol', url: 'https://github.com/univrs/univrs-dol', status: 'stable', tests: 454 },
  { name: 'univrs-enr', url: 'https://github.com/univrs/univrs-enr', status: 'active', tests: 0 },
  { name: 'univrs-network', url: 'https://github.com/univrs/univrs-network', status: 'pending', tests: 0 },
  { name: 'univrs-vudo', url: 'https://github.com/univrs/univrs-vudo', status: 'stable', tests: 402 },
];

const enrSubsystems: ENRSubsystem[] = [
  { name: 'Core', status: 'active', dolLines: 529, formula: 'Credits, NodeId, CreditTransfer' },
  { name: 'Entropy', status: 'pending', dolLines: 405, formula: 'S = wₙ·Sₙ + wᶜ·Sᶜ + wˢ·Sˢ + wᵗ·Sᵗ' },
  { name: 'Nexus', status: 'pending', dolLines: 525, formula: 'Election, Gradient Aggregation' },
  { name: 'Revival', status: 'pending', dolLines: 521, formula: '40% / 25% / 20% / 15%' },
  { name: 'Septal', status: 'pending', dolLines: 463, formula: 'Circuit Breaker, Woronin' },
  { name: 'Pricing', status: 'pending', dolLines: 651, formula: 'Fixed / Dynamic / Auction' },
];

// ═══════════════════════════════════════════════════════════════════════════
// SVG COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

const MyceliumSVG = ({ className }: { className?: string }) => (
  <svg
    viewBox="0 0 400 100"
    className={className}
    style={{ width: '100%', height: 'auto', opacity: 0.15 }}
  >
    <defs>
      <linearGradient id="myceliumGrad" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" stopColor="#00ff88" stopOpacity="0.8" />
        <stop offset="50%" stopColor="#00ffcc" stopOpacity="0.6" />
        <stop offset="100%" stopColor="#00ff88" stopOpacity="0.8" />
      </linearGradient>
    </defs>
    {/* Main hyphal network */}
    <path
      d="M0,50 Q50,30 100,50 T200,50 T300,50 T400,50"
      stroke="url(#myceliumGrad)"
      strokeWidth="2"
      fill="none"
    >
      <animate attributeName="d" dur="8s" repeatCount="indefinite"
        values="M0,50 Q50,30 100,50 T200,50 T300,50 T400,50;
                M0,50 Q50,70 100,50 T200,50 T300,50 T400,50;
                M0,50 Q50,30 100,50 T200,50 T300,50 T400,50" />
    </path>
    {/* Branching hyphae */}
    <path d="M100,50 Q120,20 150,30" stroke="url(#myceliumGrad)" strokeWidth="1" fill="none" opacity="0.6" />
    <path d="M200,50 Q180,80 160,70" stroke="url(#myceliumGrad)" strokeWidth="1" fill="none" opacity="0.6" />
    <path d="M300,50 Q320,25 350,35" stroke="url(#myceliumGrad)" strokeWidth="1" fill="none" opacity="0.6" />
    {/* Node points */}
    <circle cx="100" cy="50" r="4" fill="#00ff88" opacity="0.8">
      <animate attributeName="r" dur="2s" repeatCount="indefinite" values="3;5;3" />
    </circle>
    <circle cx="200" cy="50" r="4" fill="#00ffcc" opacity="0.8">
      <animate attributeName="r" dur="2.5s" repeatCount="indefinite" values="3;5;3" />
    </circle>
    <circle cx="300" cy="50" r="4" fill="#00ff88" opacity="0.8">
      <animate attributeName="r" dur="3s" repeatCount="indefinite" values="3;5;3" />
    </circle>
  </svg>
);

const StatusIcon = ({ status }: { status: 'complete' | 'active' | 'pending' | 'stable' }) => {
  const colors = {
    complete: '#00ff88',
    stable: '#00ff88',
    active: '#ffcc00',
    pending: '#666666',
  };
  
  return (
    <svg width="16" height="16" viewBox="0 0 16 16">
      <circle cx="8" cy="8" r="6" fill={colors[status]} opacity="0.3" />
      <circle cx="8" cy="8" r="4" fill={colors[status]}>
        {status === 'active' && (
          <animate attributeName="opacity" dur="1s" repeatCount="indefinite" values="1;0.5;1" />
        )}
      </circle>
    </svg>
  );
};

// ═══════════════════════════════════════════════════════════════════════════
// MAIN COMPONENT
// ═══════════════════════════════════════════════════════════════════════════

export const MilestoneTracker = () => {
  const [activeTab, setActiveTab] = useState<'phases' | 'repos' | 'enr'>('phases');
  const [totalTests, setTotalTests] = useState(0);

  useEffect(() => {
    const total = phases.reduce((sum, p) => sum + p.tests, 0);
    setTotalTests(total);
  }, []);

  const completedPhases = phases.filter(p => p.status === 'complete').length;
  const totalDolLines = enrSubsystems.reduce((sum, s) => sum + s.dolLines, 0);

  return (
    <section className="milestone-tracker" style={styles.container}>
      {/* Background SVG */}
      <div style={styles.backgroundSvg}>
        <MyceliumSVG />
      </div>

      {/* Header */}
      <header style={styles.header}>
        <h2 style={styles.title}>
          <span style={styles.emoji}>🍄</span> Development Milestones
        </h2>
        <p style={styles.subtitle}>
          The network is not pipes. It is a living market.
        </p>
      </header>

      {/* Stats Bar */}
      <div style={styles.statsBar}>
        <div style={styles.stat}>
          <span style={styles.statValue}>{completedPhases}/{phases.length}</span>
          <span style={styles.statLabel}>Phases</span>
        </div>
        <div style={styles.stat}>
          <span style={styles.statValue}>{totalTests.toLocaleString()}</span>
          <span style={styles.statLabel}>Tests</span>
        </div>
        <div style={styles.stat}>
          <span style={styles.statValue}>{totalDolLines.toLocaleString()}</span>
          <span style={styles.statLabel}>DOL Lines</span>
        </div>
      </div>

      {/* Tab Navigation */}
      <nav style={styles.tabs}>
        {(['phases', 'repos', 'enr'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              ...styles.tab,
              ...(activeTab === tab ? styles.tabActive : {}),
            }}
          >
            {tab === 'phases' && '📦 Phases'}
            {tab === 'repos' && '📁 Repos'}
            {tab === 'enr' && '⚡ ENR'}
          </button>
        ))}
      </nav>

      {/* Content */}
      <div style={styles.content}>
        {activeTab === 'phases' && (
          <div style={styles.phaseList}>
            {phases.map(phase => (
              <div key={phase.id} style={styles.phaseItem}>
                <div style={styles.phaseHeader}>
                  <StatusIcon status={phase.status} />
                  <span style={styles.phaseName}>{phase.name}</span>
                  <span style={styles.phaseTests}>{phase.tests} tests</span>
                </div>
                <p style={styles.phaseDesc}>{phase.description}</p>
                <div style={styles.deliverables}>
                  {phase.deliverables.map((d, i) => (
                    <span key={i} style={styles.deliverable}>{d}</span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        {activeTab === 'repos' && (
          <div style={styles.repoGrid}>
            {repositories.map(repo => (
              <a
                key={repo.name}
                href={repo.url}
                target="_blank"
                rel="noopener noreferrer"
                style={styles.repoCard}
              >
                <StatusIcon status={repo.status} />
                <span style={styles.repoName}>{repo.name}</span>
                <span style={styles.repoTests}>{repo.tests} tests</span>
              </a>
            ))}
          </div>
        )}

        {activeTab === 'enr' && (
          <div style={styles.enrGrid}>
            {enrSubsystems.map(sys => (
              <div key={sys.name} style={styles.enrCard}>
                <div style={styles.enrHeader}>
                  <StatusIcon status={sys.status} />
                  <span style={styles.enrName}>{sys.name}</span>
                </div>
                <code style={styles.enrFormula}>{sys.formula}</code>
                <span style={styles.enrLines}>{sys.dolLines} DOL lines</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <footer style={styles.footer}>
        <span>Last updated: {new Date().toLocaleDateString()}</span>
        <span style={styles.footerEmoji}>🌿</span>
      </footer>
    </section>
  );
};

// ═══════════════════════════════════════════════════════════════════════════
// STYLES (CSS-in-JS matching Univrs aesthetic)
// ═══════════════════════════════════════════════════════════════════════════

const styles: Record<string, React.CSSProperties> = {
  container: {
    position: 'relative',
    background: 'linear-gradient(135deg, #0a0f0a 0%, #0f1a0f 50%, #0a0f0a 100%)',
    borderRadius: '16px',
    padding: '2rem',
    fontFamily: "'Inter', 'SF Pro', -apple-system, sans-serif",
    color: '#e0e0e0',
    overflow: 'hidden',
    border: '1px solid rgba(0, 255, 136, 0.2)',
  },
  backgroundSvg: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    pointerEvents: 'none',
    zIndex: 0,
  },
  header: {
    position: 'relative',
    zIndex: 1,
    textAlign: 'center',
    marginBottom: '1.5rem',
  },
  title: {
    fontSize: '1.5rem',
    fontWeight: 600,
    color: '#ffffff',
    margin: 0,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '0.5rem',
  },
  emoji: {
    fontSize: '1.75rem',
  },
  subtitle: {
    fontSize: '0.875rem',
    color: '#00ff88',
    fontStyle: 'italic',
    marginTop: '0.5rem',
    opacity: 0.8,
  },
  statsBar: {
    position: 'relative',
    zIndex: 1,
    display: 'flex',
    justifyContent: 'center',
    gap: '2rem',
    marginBottom: '1.5rem',
    padding: '1rem',
    background: 'rgba(0, 255, 136, 0.05)',
    borderRadius: '8px',
  },
  stat: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
  },
  statValue: {
    fontSize: '1.5rem',
    fontWeight: 700,
    color: '#00ff88',
  },
  statLabel: {
    fontSize: '0.75rem',
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
  },
  tabs: {
    position: 'relative',
    zIndex: 1,
    display: 'flex',
    justifyContent: 'center',
    gap: '0.5rem',
    marginBottom: '1.5rem',
  },
  tab: {
    padding: '0.5rem 1rem',
    background: 'transparent',
    border: '1px solid rgba(0, 255, 136, 0.3)',
    borderRadius: '20px',
    color: '#888',
    cursor: 'pointer',
    fontSize: '0.875rem',
    transition: 'all 0.2s',
  },
  tabActive: {
    background: 'rgba(0, 255, 136, 0.15)',
    borderColor: '#00ff88',
    color: '#00ff88',
  },
  content: {
    position: 'relative',
    zIndex: 1,
  },
  phaseList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1rem',
  },
  phaseItem: {
    background: 'rgba(255, 255, 255, 0.03)',
    borderRadius: '8px',
    padding: '1rem',
    border: '1px solid rgba(255, 255, 255, 0.05)',
  },
  phaseHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    marginBottom: '0.5rem',
  },
  phaseName: {
    flex: 1,
    fontWeight: 500,
    color: '#ffffff',
  },
  phaseTests: {
    fontSize: '0.75rem',
    color: '#00ff88',
    background: 'rgba(0, 255, 136, 0.1)',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
  },
  phaseDesc: {
    fontSize: '0.875rem',
    color: '#888',
    margin: '0 0 0.75rem 0',
  },
  deliverables: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '0.5rem',
  },
  deliverable: {
    fontSize: '0.75rem',
    background: 'rgba(255, 255, 255, 0.05)',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    color: '#aaa',
  },
  repoGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '1rem',
  },
  repoCard: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    background: 'rgba(255, 255, 255, 0.03)',
    padding: '1rem',
    borderRadius: '8px',
    textDecoration: 'none',
    color: 'inherit',
    border: '1px solid rgba(255, 255, 255, 0.05)',
    transition: 'all 0.2s',
  },
  repoName: {
    flex: 1,
    fontWeight: 500,
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '0.875rem',
  },
  repoTests: {
    fontSize: '0.75rem',
    color: '#00ff88',
  },
  enrGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
    gap: '1rem',
  },
  enrCard: {
    background: 'rgba(255, 255, 255, 0.03)',
    padding: '1rem',
    borderRadius: '8px',
    border: '1px solid rgba(255, 255, 255, 0.05)',
  },
  enrHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    marginBottom: '0.5rem',
  },
  enrName: {
    fontWeight: 600,
    color: '#ffffff',
  },
  enrFormula: {
    display: 'block',
    fontSize: '0.75rem',
    color: '#00ffcc',
    background: 'rgba(0, 255, 136, 0.05)',
    padding: '0.5rem',
    borderRadius: '4px',
    marginBottom: '0.5rem',
    fontFamily: "'JetBrains Mono', monospace",
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  enrLines: {
    fontSize: '0.75rem',
    color: '#666',
  },
  footer: {
    position: 'relative',
    zIndex: 1,
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    gap: '0.5rem',
    marginTop: '1.5rem',
    paddingTop: '1rem',
    borderTop: '1px solid rgba(255, 255, 255, 0.05)',
    fontSize: '0.75rem',
    color: '#666',
  },
  footerEmoji: {
    fontSize: '1rem',
  },
};

export default MilestoneTracker;
