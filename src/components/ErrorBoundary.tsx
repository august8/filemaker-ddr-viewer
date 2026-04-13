import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  /** この値が変わると自動リセット（ナビゲーション時に別の画面へ移動した際に回復させる） */
  resetKey?: unknown;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidUpdate(prevProps: Props) {
    if (this.state.error && prevProps.resetKey !== this.props.resetKey) {
      this.setState({ error: null });
    }
  }

  private handleCopy() {
    const { error } = this.state;
    if (!error) return;
    const text = [
      `エラー: ${error.message}`,
      ``,
      `スタック:`,
      error.stack ?? "(スタックなし)",
    ].join("\n");
    navigator.clipboard.writeText(text);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;

    return (
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="max-w-lg w-full bg-white border border-red-200 rounded-lg p-6 space-y-4">
          <h2 className="text-base font-semibold text-red-700">
            エラーが発生しました
          </h2>
          <p className="text-sm text-gray-600">
            別の項目を選択すると自動的に回復します。
            エラーが繰り返す場合は、以下の情報を開発者に共有してください。
          </p>
          <div className="bg-gray-50 border border-gray-200 rounded p-3 text-xs font-mono text-gray-700 break-all max-h-40 overflow-y-auto">
            {error.message}
          </div>
          <button
            onClick={() => this.handleCopy()}
            className="px-3 py-1.5 text-xs font-medium rounded border border-gray-300
                       bg-white text-gray-700 hover:bg-gray-50 transition-colors"
          >
            エラー情報をクリップボードにコピー
          </button>
        </div>
      </div>
    );
  }
}
