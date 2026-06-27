import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';

const rootDir = process.cwd();

function getLatestCommitDate() {
  try {
    return execSync('git log -1 --format=%cI', { encoding: 'utf8' }).trim();
  } catch (e) {
    return new Date().toISOString();
  }
}

function bumpVersion(current, releaseType) {
  const [major, minor, patch] = current.split('.').map(Number);
  if (releaseType === 'major') return `${major + 1}.0.0`;
  if (releaseType === 'minor') return `${major}.${minor + 1}.0`;
  if (releaseType === 'patch') return `${major}.${minor}.${patch + 1}`;
  if (/^\d+\.\d+\.\d+$/.test(releaseType)) return releaseType;
  throw new Error(`Invalid release type or version: ${releaseType}`);
}

function updatePackageJson(newVersion) {
  const pPath = path.join(rootDir, 'package.json');
  const p = JSON.parse(fs.readFileSync(pPath, 'utf8'));
  const oldVersion = p.version;
  p.version = newVersion;
  fs.writeFileSync(pPath, JSON.stringify(p, null, 2) + '\n', 'utf8');
  console.log(`Updated package.json: ${oldVersion} -> ${newVersion}`);
}

function updateCargoToml(newVersion) {
  const cPath = path.join(rootDir, 'src-tauri/Cargo.toml');
  if (!fs.existsSync(cPath)) return;
  let content = fs.readFileSync(cPath, 'utf8');
  const oldContent = content;
  
  // Replace version under [package]
  content = content.replace(/(\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m, `$1${newVersion}$2`);
  if (content === oldContent) {
    // fallback generic replacement (with multiline flag)
    content = content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
  }
  fs.writeFileSync(cPath, content, 'utf8');
  console.log(`Updated src-tauri/Cargo.toml version to ${newVersion}`);
}

function updateTauriConf(newVersion) {
  const tPath = path.join(rootDir, 'src-tauri/tauri.conf.json');
  if (!fs.existsSync(tPath)) return;
  try {
    const t = JSON.parse(fs.readFileSync(tPath, 'utf8'));
    if (t.package && t.package.version !== undefined) {
      const oldVersion = t.package.version;
      t.package.version = newVersion;
      fs.writeFileSync(tPath, JSON.stringify(t, null, 2) + '\n', 'utf8');
      console.log(`Updated src-tauri/tauri.conf.json: ${oldVersion} -> ${newVersion}`);
    }
  } catch (e) {
    console.warn(`Failed to update src-tauri/tauri.conf.json: ${e.message}`);
  }
}

function updateChangelog(newVersion) {
  const clPath = path.join(rootDir, 'CHANGELOG.md');
  if (!fs.existsSync(clPath)) return;
  let content = fs.readFileSync(clPath, 'utf8');
  
  // Find ## [Unreleased] section
  const unreleasedRegex = /(##\s*\[Unreleased\])([\s\S]*?)(?=##\s*\[\d+\.\d+\.\d+\]|#\s*Changelog|$)/i;
  const match = content.match(unreleasedRegex);
  
  if (!match) {
    throw new Error('Could not find ## [Unreleased] section in CHANGELOG.md');
  }
  
  const header = match[1];
  const unreleasedContent = match[2].trim();
  const commitDate = getLatestCommitDate();
  
  // Prepare new Unreleased and the bumped version section with proper spacing
  const newSection = `${header}\n\n## [${newVersion}] - ${commitDate}\n\n${unreleasedContent ? unreleasedContent + '\n\n' : ''}`;
  
  // Replace the old ## [Unreleased] section
  const updatedContent = content.replace(unreleasedRegex, newSection);
  fs.writeFileSync(clPath, updatedContent, 'utf8');
  console.log(`Updated CHANGELOG.md with version ${newVersion} and date ${commitDate}`);
}

function main() {
  const releaseType = process.argv[2];
  if (!releaseType) {
    console.error('Usage: node release.js <patch|minor|major|x.y.z>');
    process.exit(1);
  }

  const pPath = path.join(rootDir, 'package.json');
  const p = JSON.parse(fs.readFileSync(pPath, 'utf8'));
  const currentVersion = p.version;
  
  const newVersion = bumpVersion(currentVersion, releaseType);
  console.log(`Bumping version from ${currentVersion} to ${newVersion}...`);

  updatePackageJson(newVersion);
  updateTauriConf(newVersion);
  updateCargoToml(newVersion);
  updateChangelog(newVersion);
  
  // Sync lockfiles
  console.log('Syncing package-lock.json...');
  execSync('npm install --package-lock-only', { stdio: 'inherit' });
  
  const cargoLockPath = path.join(rootDir, 'src-tauri/Cargo.lock');
  if (fs.existsSync(cargoLockPath)) {
    console.log('Syncing src-tauri/Cargo.lock...');
    execSync('cargo check --manifest-path src-tauri/Cargo.toml', { stdio: 'inherit' });
  }

  console.log(`\nVersion bump to ${newVersion} completed successfully!`);
  console.log('Please verify the changes, run tests, and commit/tag when ready.');
}

main();
