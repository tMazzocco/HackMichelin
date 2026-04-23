interface Props {
  size?: number;
  color?: string;
  style?: React.CSSProperties;
  className?: string;
}

export default function MichelinStar({ size = 24, color = "currentColor", style, className }: Props) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill={color}
      style={style}
      className={className}
    >
      {/*
        6-petal Michelin star.
        Each petal is a circular arc (r=6) centred at distance 6 from origin.
        Petal centres at 270°/330°/30°/90°/150°/210°.
        Adjacent petals intersect at radius ≈10.39 (outer points) and at
        the figure centre (12,12). Path traces the outer boundary CCW.
      */}
      <path d="M17.2 3 A6 6 0 0 0 6.8 3 A6 6 0 0 0 1.61 12 A6 6 0 0 0 6.8 21 A6 6 0 0 0 17.2 21 A6 6 0 0 0 22.39 12 A6 6 0 0 0 17.2 3 Z" />
    </svg>
  );
}
