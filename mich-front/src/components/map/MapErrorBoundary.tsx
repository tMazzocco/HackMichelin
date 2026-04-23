import { Component, ReactNode } from "react";
import { MapPin } from "lucide-react";

interface Props {
  children: ReactNode;
  height?: string;
}

interface State {
  crashed: boolean;
}

export default class MapErrorBoundary extends Component<Props, State> {
  state: State = { crashed: false };

  static getDerivedStateFromError(): State {
    return { crashed: true };
  }

  componentDidCatch(err: Error) {
    console.error("[MapErrorBoundary]", err.message);
  }

  render() {
    if (this.state.crashed) {
      return (
        <div
          className="w-full flex flex-col items-center justify-center gap-2 bg-black/5 rounded-2xl text-text/40"
          style={{ height: this.props.height ?? "100%" }}
        >
          <MapPin size={24} className="text-text/20" />
          <p className="text-xs">Map unavailable</p>
        </div>
      );
    }
    return this.props.children;
  }
}
