/**
 * This file contains intentional style warnings to demonstrate starlint's
 * style and suggestion rules. Run `npm run lint` to see the diagnostics.
 */

function fetchData(url) {
  // starlint: no-console (warn)
  console.log("Fetching:", url);

  return fetch(url).then(function (response) {
    console.log("Got response");
    return response.json();
  });
}

module.exports = { fetchData };
