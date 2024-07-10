export function isValidU32(value: number, minimum: number = 0): boolean {
  return (
    !isNaN(value) && isFinite(value) && value >= minimum && value < 2 ** 32
  );
}
