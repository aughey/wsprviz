// OpenLayers 10.7.0 is served locally from vendor/openlayers.
const maps = new Map();
let nextId = 0;

export function gridCenter(locator) {
  if (typeof locator !== "string") return null;
  const grid = locator.trim().toUpperCase();
  if (!/^[A-R]{2}(?:[0-9]{2}(?:[A-X]{2}(?:[0-9]{2})?)?)?$/.test(grid)) {
    return null;
  }
  let longitude = -180;
  let latitude = -90;
  let width = 20;
  let height = 10;
  longitude += (grid.charCodeAt(0) - 65) * width;
  latitude += (grid.charCodeAt(1) - 65) * height;
  for (let index = 2; index < grid.length; index += 2) {
    const numeric = index % 4 === 2;
    const divisions = numeric ? 10 : 24;
    const offset = numeric ? 48 : 65;
    width /= divisions;
    height /= divisions;
    longitude += (grid.charCodeAt(index) - offset) * width;
    latitude += (grid.charCodeAt(index + 1) - offset) * height;
  }
  return [longitude + width / 2, latitude + height / 2];
}

export function createMap(target, reports, transmitted) {
  const ol = window.ol;
  const container = document.getElementById(target);
  const stations = new Map();
  let skipped = 0;
  for (const report of JSON.parse(reports)) {
    for (const role of ["rx", "tx"]) {
      const callsign = report[role + "_sign"];
      const grid = report[role + "_loc"].trim().toUpperCase();
      const coordinates = gridCenter(grid);
      if (!coordinates) {
        skipped++;
        continue;
      }
      const key = JSON.stringify([role, callsign, grid]);
      stations.set(key, { callsign, grid, role, coordinates });
    }
  }

  const homeRole = transmitted ? "tx" : "rx";
  const homeStyle = new ol.style.Style({
    image: new ol.style.RegularShape({
      points: 5, radius: 13, radius2: 6,
      fill: new ol.style.Fill({ color: "#115d50" }),
      stroke: new ol.style.Stroke({ color: "white", width: 2 }),
    }),
    zIndex: 2,
  });
  const remoteStyle = new ol.style.Style({
    image: new ol.style.Circle({
      radius: 6,
      fill: new ol.style.Fill({ color: "#be5423" }),
      stroke: new ol.style.Stroke({ color: "white", width: 2 }),
    }),
    zIndex: 1,
  });
  const features = Array.from(stations.values(), station => {
    // Web Mercator has a finite latitude range.
    const [longitude, latitude] = station.coordinates;
    const feature = new ol.Feature({
      ...station,
      geometry: new ol.geom.Point(ol.proj.fromLonLat([
        longitude, Math.max(-85.05112878, Math.min(85.05112878, latitude)),
      ])),
    });
    feature.setStyle(station.role === homeRole ? homeStyle : remoteStyle);
    return feature;
  });
  const source = new ol.source.Vector({ features, wrapX: false });
  const popup = document.createElement("div");
  popup.className = "map-popup";
  popup.setAttribute("role", "status");
  container.append(popup);
  const overlay = new ol.Overlay({ element: popup, positioning: "bottom-center", offset: [0, -16] });
  const tileStatus = document.createElement("p");
  tileStatus.className = "map-help";
  tileStatus.setAttribute("role", "status");
  container.after(tileStatus);
  if (skipped) {
    tileStatus.textContent = "Some reports have invalid grid locations and could not be mapped.";
  }
  if (!features.length) {
    tileStatus.textContent = "These reports have no valid grid locations to map.";
  }
  const tiles = new ol.source.OSM();
  tiles.on("tileloaderror", () => {
    tileStatus.textContent = "Some OpenStreetMap tiles could not load. Station markers are still available.";
  });
  const map = new ol.Map({
    target: container,
    layers: [new ol.layer.Tile({ source: tiles }), new ol.layer.Vector({ source })],
    overlays: [overlay],
    view: new ol.View({ center: [0, 0], zoom: 2, showFullExtent: true }),
  });
  map.on("singleclick", event => {
    const descriptions = new Set();
    map.forEachFeatureAtPixel(event.pixel, feature => {
      descriptions.add(`${feature.get("role") === "rx" ? "Receiver" : "Transmitter"} ${feature.get("callsign")} · ${feature.get("grid")}`);
    }, { hitTolerance: 5 });
    popup.textContent = Array.from(descriptions).join("\n");
    overlay.setPosition(descriptions.size ? event.coordinate : undefined);
  });
  map.on("pointermove", event => {
    container.style.cursor = map.hasFeatureAtPixel(event.pixel) ? "pointer" : "";
  });
  const resize = new ResizeObserver(() => {
    map.updateSize();
    if (features.length) {
      map.getView().fit(source.getExtent(), { padding: [45, 45, 45, 45], maxZoom: 10 });
    }
  });
  resize.observe(container);
  const id = ++nextId;
  maps.set(id, { map, resize, tileStatus, popup });
  return id;
}

export function disposeMap(id) {
  const entry = maps.get(id);
  if (!entry) return;
  entry.resize.disconnect();
  entry.map.setTarget(undefined);
  entry.map.dispose();
  entry.tileStatus.remove();
  entry.popup.remove();
  maps.delete(id);
}
