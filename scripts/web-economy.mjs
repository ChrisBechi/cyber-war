// Bounds in the campaign's fictional economy, in cents. Unclassified collectible objects
// retain their authored price; known everyday product families must stay in these ranges.
export const PriceModel = {
  usb: [500, 50000],
  phone: [30000, 900000],
  router: [5000, 250000],
  gpu: [20000, 2000000],
  chair: [2000, 500000],
  'rubber-duck': [100, 20000],
};
export function validatePrice(document, price) {
  for (const entity of document.entities) {
    const range = PriceModel[entity];
    if (range && (price < range[0] || price > range[1])) {
      throw new Error(`Price outside ${entity} economy: ${document.id} (${price})`);
    }
  }
}
