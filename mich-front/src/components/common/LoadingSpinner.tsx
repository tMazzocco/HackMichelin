import { Loader } from "@mantine/core";

export default function LoadingSpinner({ size = 24 }: { size?: number }) {
  return <Loader color="michelin" size={size} />;
}
