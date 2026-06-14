// specite postinstall — downloads the platform-correct pre-built binary.
import {
  createWriteStream,
  existsSync,
  mkdirSync,
  renameSync,
  chmodSync,
  rmSync,
  readFileSync,
} from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { spawnSync } from "child_process";
import https from "https";
import http from "http";

const __dirname = dirname(fileURLToPath(import.meta.url));

const PKG = "specite";
const REPO = "fxcl/opencode-specite";

// ── Platform detection ────────────────────────────────────────────
function getTarget() {
  const platform = process.platform;
  const arch = process.arch;

  const osMap = {
    darwin: "darwin",
    linux: "linux",
    win32: "windows",
  };

  const archMap = {
    x64: "x64",
    arm64: "arm64",
  };

  const osLabel = osMap[platform];
  const archLabel = archMap[arch];

  if (!osLabel || !archLabel) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
        `Supported: darwin-x64, darwin-arm64, linux-x64, linux-arm64, windows-x64.`,
    );
  }

  return `${osLabel}-${archLabel}`;
}

// ── Version ───────────────────────────────────────────────────────
function getVersion() {
  const pkgPath = join(__dirname, "package.json");
  return JSON.parse(readFileSync(pkgPath, "utf8")).version;
}

// ── Download helper ───────────────────────────────────────────────
function downloadFile(url, dest, version) {
  return new Promise((resolve, reject) => {
    const attempt = (url, redirects = 0) => {
      if (redirects > 5) {
        reject(new Error("Too many redirects"));
        return;
      }

      const lib = url.startsWith("https") ? https : http;
      const req = lib.get(url, (res) => {
        if (
          res.statusCode >= 300 &&
          res.statusCode < 400 &&
          res.headers.location
        ) {
          res.resume();
          attempt(res.headers.location, redirects + 1);
          return;
        }

        if (res.statusCode !== 200) {
          res.resume();
          reject(
            new Error(
              `Download failed: HTTP ${res.statusCode} for ${url}\n` +
                `Make sure release v${version} exists at ` +
                `https://github.com/${REPO}/releases`,
            ),
          );
          return;
        }

        const stream = createWriteStream(dest);
        res.pipe(stream);
        stream.on("finish", () => {
          stream.close();
          resolve();
        });
        stream.on("error", reject);
      });

      req.on("error", reject);
      req.setTimeout(60000, () => {
        req.destroy(new Error("Download timeout"));
      });
    };

    attempt(url);
  });
}

// ── Extract helpers ───────────────────────────────────────────────
function extractTarGz(archivePath, destDir) {
  const result = spawnSync("tar", ["-xzf", archivePath, "-C", destDir], {
    stdio: "pipe",
  });

  if (result.status !== 0) {
    throw new Error(
      `Failed to extract ${archivePath}: ${result.stderr?.toString()}`,
    );
  }
}

function extractZip(archivePath, destDir) {
  // PowerShell 5.0+ (Windows 10+) has Expand-Archive
  const psResult = spawnSync(
    "powershell",
    [
      "-NoProfile",
      "-NonInteractive",
      "-Command",
      `Expand-Archive -Force -Path '${archivePath}' -DestinationPath '${destDir}'`,
    ],
    { stdio: "pipe" },
  );

  if (psResult.status !== 0) {
    throw new Error(
      `Failed to extract ${archivePath}: ${psResult.stderr?.toString()}`,
    );
  }
}

// ── Download & extract ────────────────────────────────────────────
async function downloadBinary(target, version, destDir) {
  const isWindows = target.startsWith("windows");
  const ext = isWindows ? ".exe" : "";
  const archiveExt = isWindows ? ".zip" : ".tar.gz";
  const archiveName = `${PKG}-${target}${archiveExt}`;
  const url = `https://github.com/${REPO}/releases/download/v${version}/${archiveName}`;

  console.log(`${PKG}: downloading ${archiveName} from GitHub Releases…`);

  const tmpDir = join(destDir, ".download");
  mkdirSync(tmpDir, { recursive: true });
  const tmpArchive = join(tmpDir, archiveName);

  // Download
  await downloadFile(url, tmpArchive, version);

  // Extract binary from archive
  console.log(`${PKG}: extracting…`);
  if (isWindows) {
    extractZip(tmpArchive, tmpDir);
  } else {
    extractTarGz(tmpArchive, tmpDir);
  }

  // Move binary to destination
  const binaryName = `${PKG}${ext}`;
  const extractedPath = join(tmpDir, binaryName);
  const finalPath = join(destDir, binaryName);

  if (!existsSync(extractedPath)) {
    throw new Error(
      `Binary ${binaryName} not found in archive. ` +
        `Archive may be corrupted or have unexpected structure.`,
    );
  }

  renameSync(extractedPath, finalPath);

  // Make executable (Unix only)
  if (!isWindows) {
    chmodSync(finalPath, 0o755);
  }

  // Cleanup temp
  rmSync(tmpDir, { recursive: true, force: true });

  console.log(`${PKG}: installed v${version} (${target})`);
}

// ── Main ──────────────────────────────────────────────────────────
async function main() {
  // Respect npm config to skip binary download (e.g. for packaging)
  if (process.env.npm_config_specite_skip_download === "true") {
    console.log(`${PKG}: skipping binary download (skip_download set)`);
    return;
  }

  const target = getTarget();
  const version = getVersion();

  // Binaries live alongside the JS wrapper in bin/
  const binDir = join(__dirname, "bin");
  mkdirSync(binDir, { recursive: true });

  // Skip if already present (npm rebuild)
  const ext = target.startsWith("windows") ? ".exe" : "";
  if (existsSync(join(binDir, "specite" + ext))) {
    console.log(`${PKG}: binary already present, skipping download.`);
    return;
  }

  try {
    await downloadBinary(target, version, binDir);
  } catch (err) {
    console.error(`${PKG}: ${err.message}`);
    console.error(
      `\n${PKG}: You can still build from source:\n` +
        `  git clone https://github.com/${REPO}\n` +
        `  cd opencode-specite\n` +
        `  cargo install --path .\n`,
    );
    process.exit(1);
  }
}

main();
