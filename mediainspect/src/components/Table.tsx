interface TableProps<T> {
  rows: T[];
  columns: { header: string; cell: (row: T) => React.ReactNode; className?: string }[];
}

export function Table<T>({ rows, columns }: TableProps<T>) {
  return (
    <div className="overflow-auto rounded border border-line">
      <table className="w-full text-sm">
        <thead className="bg-panel">
          <tr>
            {columns.map((col) => (
              <th
                key={col.header}
                className="text-left font-medium text-muted px-3 py-2 text-[11px] uppercase tracking-wide"
              >
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="font-mono">
          {rows.map((row, i) => (
            <tr key={i} className="border-t border-line">
              {columns.map((col) => (
                <td key={col.header} className={`px-3 py-2 align-top ${col.className ?? ""}`}>
                  {col.cell(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
