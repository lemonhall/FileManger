export function formatSize(size) {
    if (size === null || size === undefined) return '-';
    const numSize = Number(size); // Ensure it's a number
    if (isNaN(numSize)) return '-';
    if (numSize < 1024) return `${numSize} B`;
    if (numSize < 1024 * 1024) return `${(numSize / 1024).toFixed(1)} KB`;
    if (numSize < 1024 * 1024 * 1024) return `${(numSize / (1024 * 1024)).toFixed(1)} MB`;
    return `${(numSize / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function vipTypeToString(vipType) {
  switch (vipType) {
    case 0: return '普通用户';
    case 1: return '普通会员';
    case 2: return '超级会员';
    default: return `未知 (${vipType})`;
  }
} 