export default function LoadingSpinner({ size = 24 }: { size?: number }) {
  return (
    <div
      className="animate-spin rounded-full border-2 border-primary/20 border-t-primary"
      style={{ width: size, height: size }}
    />
  );
}
