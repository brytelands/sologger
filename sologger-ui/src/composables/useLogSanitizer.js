// Collapses newlines/tabs and runs of whitespace so log payloads render as single lines.
// Objects are stringified with the same normalization applied to every string value.
export function sanitizeLogMessage(message) {
  if (typeof message === 'object') {
    return JSON.stringify(message, (key, value) => {
      if (typeof value === 'string') {
        return value.replace(/[\n\r\t]/g, ' ').replace(/\s+/g, ' ');
      }
      return value;
    });
  }
  if (typeof message === 'string') {
    return message.replace(/[\n\r\t]/g, ' ').replace(/\s+/g, ' ');
  }
  return message;
}
