import { Component, type ErrorInfo, type ReactNode } from "react";

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

  componentDidCatch(error: Error, _info: ErrorInfo) {
    this.props.onError(error.message || "Unexpected render error");
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="app-fallback">
          <h2>Something went wrong.</h2>
          <p>Reload the app to recover from this error.</p>
        </div>
      );
    }
    return this.props.children;
  }
}
