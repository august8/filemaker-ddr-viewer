// src/components/RightPanel.tsx

interface Props {
  title: string;
  onClose: () => void;
  children: React.ReactNode;
  width?: number;
}

export function RightPanel({ title, onClose, children, width }: Props) {
  return (
    <div className="border-l border-gray-200 bg-white overflow-auto flex flex-col shrink-0" style={{ width: width ?? 288 }}>
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-gray-50 shrink-0">
        <h3 className="font-semibold text-sm text-gray-700">{title}</h3>
        <button
          onClick={onClose}
          className="text-gray-400 hover:text-gray-700 text-lg leading-none"
        >
          ×
        </button>
      </div>
      <div className="flex-1 overflow-auto">
        {children}
      </div>
    </div>
  );
}
