import fs from "fs";
import https from "https";
import path from "path";
import { fileURLToPath } from "url";

// ESM doesn't support __dirname and __filename by default
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const packageJson = JSON.parse(fs.readFileSync("./package.json", "utf8"));
// The version number is expected to be in parity with the ttyper release version
const { version, releasesUrl, name, binDir: directory } = packageJson;

// All the binary files will be stored in the /bin directory
const binDir = path.join(__dirname, directory);
console.log(`Installing ${name} v${version}`);

try {
  void install();
} catch (error) {
  console.error("Installation failed:", error.message);
}

async function install() {
  if (fs.existsSync(binDir)) {
    fs.rmSync(binDir, { recursive: true });
  }
  fs.mkdirSync(binDir, {
    mode: 0o777,
  });
  await getBinary();

  // Remove the node_modules as we'll only need the binary
  fs.rmSync(path.join(__dirname, "node_modules"), { recursive: true });
}

function getBinaryDownloadURL() {
  let os, arch;

  // Possible values are : 'aix' | 'android' | 'darwin' | 'freebsd' | 'haiku' | 'linux' | 'openbsd' | 'sunos' | 'win32' | 'cygwin' | 'netbsd'
  switch (process.platform) {
    case "win32":
    case "cygwin":
      os = "pc-windows-msvc";
      break;
    case "darwin":
      os = "apple-darwin";
      break;
    case "linux":
      os = "unknown-linux-gnu";
      break;
    default:
      throw new Error(`Unsupported OS: ${process.platform}`);
  }

  // Possible values are: 'arm' | 'arm64' | 'ia32' | 'mips' | 'mipsel' | 'ppc' | 'ppc64' | 's390' | 's390x' | 'x64'
  switch (process.arch) {
    case "x64":
      arch = "x86_64";
      break;
    case "arm64":
      arch = "aarch64";
      break;
    case "ia32":
      arch = "i686";
      break;
    default:
      throw new Error(`Unsupported architecture: ${process.arch}`);
  }

  const extension = os === "pc-windows-msvc" ? "zip" : "tar.gz";

  return `${releasesUrl}/download/v${version}/${name}-${arch}-${os}.${extension}`;
}

function downloadPackage(url, outputPath) {
  // We use https.get instead of fetch to get a readable stream from the response without additional dependencies
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        // If the response is a redirect, we download the package from the new location
        if (response.statusCode === 302) {
          resolve(downloadPackage(response.headers.location, outputPath));
        } else if (response.statusCode === 200) {
          const file = fs.createWriteStream(outputPath);
          response.pipe(file);
          file.on("finish", () => {
            file.close(resolve);
          });
        } else {
          reject(
            new Error(
              `Failed to download ${name}. Status code: ${response.statusCode}`
            )
          );
        }
      })
      .on("error", reject);
  });
}

async function extractPackage(inputPath, outputPath) {
  if (path.extname(inputPath) === ".gz") {
    const tar = await import("tar");
    await tar.x({
      file: inputPath,
      cwd: outputPath,
    });
  } else if (path.extname(inputPath) === ".zip") {
    const AdmZip = (await import("adm-zip")).default;
    const zip = new AdmZip(inputPath);
    zip.extractAllTo(outputPath, true, true);
  }
}

async function getBinary() {
  const downloadURL = getBinaryDownloadURL();
  console.log(`Downloading ${name} from ${downloadURL}`);

  const pkgName = ["win32", "cygwin"].includes(process.platform)
    ? `package.zip`
    : `package.tar.gz`;
  const packagePath = path.join(binDir, pkgName);

  await downloadPackage(downloadURL, packagePath);
  await extractPackage(packagePath, binDir);

  fs.rmSync(packagePath);
}
