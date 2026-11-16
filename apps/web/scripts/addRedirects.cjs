// relative to this file
const ROUTES = "../src/routes";
const BUILD = "../build";
const TEMPLATE_FILE_PATH = "./redirect.html";

const TITLE_REPLACE_STRING = "__TITLE__";
const PATH_REPLACE_STRING = "__REDIRECT_PATH__";

const fs = require("fs");
const path = require("path");

const redirectTemplate = fs.readFileSync(
  path.resolve(__dirname, TEMPLATE_FILE_PATH),
  { encoding: "utf8" }
);

const rootPath = path.resolve(__dirname, ROUTES);
const buildPath = path.resolve(__dirname, BUILD);
crawl(rootPath);

/**
 * @param { string } directory
 */
function crawl(directory) {
  const contents = fs.readdirSync(directory, { withFileTypes: true });

  contents.forEach((entry) => {
    if (entry.isDirectory()) {
      crawl(path.join(entry.parentPath, entry.name));
      return;
    }

    if (entry.name === "+page.svelte") {
      addRedirect(directory);
    }
  });
}

/**
 * @param { string } route
 */
function addRedirect(route) {
  const _routePath = path.relative(rootPath, route);
  const routePath = "/" + _routePath;
  const routeName = path.basename(routePath);

  if (routePath === "/") {
    return;
  }

  if (routeName.startsWith("[") && routeName.endsWith("]")) {
    // orange
    console.log(
      `\x1b[33mRedirects are not added for dynamic routes: ${routePath}\x1b[0m`
    );
    return;
  }

  const redirectPageContents = redirectTemplate
    .replaceAll(TITLE_REPLACE_STRING, `Redirecting to ${routePath}...`)
    .replaceAll(PATH_REPLACE_STRING, `${routePath}`);

  const redirectFileLocation = path.join(buildPath, _routePath + ".html");

  fs.mkdirSync(path.dirname(redirectFileLocation), { recursive: true });
  fs.writeFileSync(redirectFileLocation, redirectPageContents);

  // green
  console.log(`\x1b[32mAdded ${routePath} redirect\x1b[0m`);
}
