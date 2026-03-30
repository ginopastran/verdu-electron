const TICKET_TOTAL_WIDTH = 42;
const TICKET_QTY_WIDTH = 8;
const TICKET_PRICE_WIDTH = 9;
const TICKET_TOTAL_COL_WIDTH = 13;
const TICKET_SPACING_WIDTH = 3;
const TICKET_PRODUCT_WIDTH =
  TICKET_TOTAL_WIDTH -
  TICKET_QTY_WIDTH -
  TICKET_PRICE_WIDTH -
  TICKET_TOTAL_COL_WIDTH -
  TICKET_SPACING_WIDTH;

const normalizeWhitespace = (value: string) => value.trim().replace(/\s+/g, " ");

const splitTicketText = (value: string, width: number) => {
  const normalized = normalizeWhitespace(value);

  if (!normalized) {
    return [""];
  }

  const words = normalized.split(" ");
  const lines: string[] = [];
  let currentLine = "";

  for (const word of words) {
    if (!currentLine) {
      currentLine = word;
      continue;
    }

    const candidate = `${currentLine} ${word}`;
    if (candidate.length <= width) {
      currentLine = candidate;
      continue;
    }

    lines.push(currentLine);
    currentLine = word;
  }

  if (currentLine) {
    lines.push(currentLine);
  }

  return lines.flatMap((line) => {
    if (line.length <= width) {
      return [line];
    }

    const wrapped: string[] = [];
    for (let offset = 0; offset < line.length; offset += width) {
      wrapped.push(line.slice(offset, offset + width));
    }
    return wrapped;
  });
};

const padRight = (value: string, width: number) => value.padEnd(width, " ");
const padLeft = (value: string, width: number) => value.padStart(width, " ");

export const buildTicketColumnsHeader = () =>
  `${padRight("PRODUCTO", TICKET_PRODUCT_WIDTH)} ${padLeft(
    "CANT",
    TICKET_QTY_WIDTH
  )} ${padLeft("PRECIO", TICKET_PRICE_WIDTH)} ${padLeft(
    "TOTAL",
    TICKET_TOTAL_COL_WIDTH
  )}`;

export const buildTicketSeparator = () => "-".repeat(TICKET_TOTAL_WIDTH);

export const formatTicketPreviewLines = ({
  name,
  quantity,
  price,
  subtotal,
}: {
  name: string;
  quantity: number;
  price: number;
  subtotal: number;
}) => {
  const nameLines = splitTicketText(name, TICKET_PRODUCT_WIDTH);
  const quantityText = padLeft(quantity.toFixed(3), TICKET_QTY_WIDTH);
  const priceText = padLeft(`$${price.toFixed(2)}`, TICKET_PRICE_WIDTH);
  const subtotalText = padLeft(`$${subtotal.toFixed(2)}`, TICKET_TOTAL_COL_WIDTH);

  return nameLines.map((line, index) => {
    if (index < nameLines.length - 1) {
      return line;
    }

    return `${padRight(line, TICKET_PRODUCT_WIDTH)} ${quantityText} ${priceText} ${subtotalText}`;
  });
};
