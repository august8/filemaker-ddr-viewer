import { useAccountList, usePrivilegeSetList } from "../../hooks/security";
import { Spinner } from "../Spinner";

interface Props {
  projectId: number;
}

export function SecurityPanel({ projectId }: Props) {
  const { data: accounts, isLoading: accountsLoading } = useAccountList(projectId);
  const { data: privilegeSets, isLoading: psLoading } = usePrivilegeSetList(projectId);

  if (accountsLoading || psLoading) {
    return (
      <div className="flex items-center gap-2 p-4 text-sm text-gray-400"><Spinner className="w-4 h-4" />読み込み中...</div>
    );
  }

  return (
    <div className="p-4 space-y-6">
      {/* アカウント */}
      <section>
        <h2 className="text-sm font-semibold text-gray-700 mb-2">
          アカウント ({accounts?.length ?? 0})
        </h2>
        {!accounts || accounts.length === 0 ? (
          <p className="text-sm text-gray-400">アカウントなし</p>
        ) : (
          <div className="border border-gray-200 rounded divide-y divide-gray-100">
            {accounts.map((account) => (
              <div key={account.id} className="flex items-center justify-between px-3 py-2 text-sm">
                <span className="font-medium text-gray-800">{account.name}</span>
                <div className="flex items-center gap-2">
                  {account.privilege_set && (
                    <span className="text-xs text-gray-500">{account.privilege_set}</span>
                  )}
                  <span
                    className={`inline-block text-xs font-medium rounded px-2 py-0.5 ${
                      account.enabled
                        ? "bg-green-100 text-green-700"
                        : "bg-gray-100 text-gray-500"
                    }`}
                  >
                    {account.enabled ? "有効" : "無効"}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* 権限セット */}
      <section>
        <h2 className="text-sm font-semibold text-gray-700 mb-2">
          権限セット ({privilegeSets?.length ?? 0})
        </h2>
        {!privilegeSets || privilegeSets.length === 0 ? (
          <p className="text-sm text-gray-400">権限セットなし</p>
        ) : (
          <div className="border border-gray-200 rounded divide-y divide-gray-100">
            {privilegeSets.map((ps) => (
              <div key={ps.id} className="px-3 py-2 text-sm">
                <span className="font-medium text-gray-800">{ps.name}</span>
                {ps.comment && (
                  <p className="text-xs text-gray-500 mt-0.5">{ps.comment}</p>
                )}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
