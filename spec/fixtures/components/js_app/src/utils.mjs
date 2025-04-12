export const safeInt = (value) => {
  if (typeof value === 'number') {
    return value;
  }
  if (typeof value === 'string') {
    const parsed = parseInt(value);
    return isNaN(parsed) ? 0 : parsed;
  }
  return 0;
}
