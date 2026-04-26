import { spawnSync } from 'node:child_process';

const args = process.argv.slice(2);

if (args[0] === 'dev' || args[0] === 'build') {
  addFeature(args, 'local-whisper');
}

const result = spawnSync('cargo', ['tauri', ...args], {
  stdio: 'inherit',
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function addFeature(args, feature) {
  const equalsIndex = args.findIndex((arg) => arg.startsWith('--features='));
  if (equalsIndex >= 0) {
    const features = args[equalsIndex].slice('--features='.length).split(',');
    if (!features.includes(feature)) {
      args[equalsIndex] = `--features=${[...features, feature].join(',')}`;
    }
    return;
  }

  const flagIndex = args.findIndex((arg) => arg === '--features' || arg === '-F');
  if (flagIndex >= 0) {
    const nextIndex = flagIndex + 1;
    const features = (args[nextIndex] ?? '').split(',').filter(Boolean);
    if (!features.includes(feature)) {
      args[nextIndex] = [...features, feature].join(',');
    }
    return;
  }

  args.push('--features', feature);
}
