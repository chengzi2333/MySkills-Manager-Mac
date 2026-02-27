import { Component, type ReactNode } from "react";

type Props = {
  children: ReactNode;
  onError: (message: string) => void;
};

type State = {
  hasError: boolean;
};

export default class AppErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error) {
    this.props.onError(error.message || "Unexpected render error");
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="app-fallback">
          <h2>发生错误</h2>
          <p>请重新加载应用以恢复。</p>
        </div>
      );
    }
    return this.props.children;
  }
}
