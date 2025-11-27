import React from 'react';
import { cn } from '../../utils/cn';

export interface ColumnDef<T = any> {
  // 使用字符串 key 以支持派生列（不一定严格对应数据属性）
  key: string;
  title: string;
  render?: (value: unknown, record: T, index: number) => React.ReactNode;
  className?: string;
  headerClassName?: string;
}

interface TableProps<T = any> extends React.HTMLAttributes<HTMLTableElement> {
  data?: T[];
  columns?: ColumnDef<T>[];
  loading?: boolean;
  emptyMessage?: string;
}

const Table = <T extends Record<string, any> = any>({
  className,
  data = [],
  columns = [],
  loading = false,
  children,
  emptyMessage = '暂无数据',
  ...props
}: TableProps<T>) => {
  if (children) {
    return (
      <div className="overflow-x-auto">
        <table
          className={cn(
            'min-w-full divide-y divide-gray-200 dark:divide-gray-700',
            className
          )}
          {...props}
        >
          {children}
        </table>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table
        className={cn(
          'min-w-full divide-y divide-gray-200 dark:divide-gray-700',
          className
        )}
        {...props}
      >
        <thead className="bg-gray-50 dark:bg-gray-800">
          <tr>
            {columns.map((column) => (
              <th
                key={column.key}
                scope="col"
                className={cn(
                  'px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-400',
                  column.headerClassName
                )}
              >
                {column.title}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="bg-white divide-y divide-gray-200 dark:bg-gray-900 dark:divide-gray-700">
          {loading ? (
            <tr>
              <td
                colSpan={columns.length}
                className="px-6 py-12 text-center"
              >
                <div className="flex justify-center">
                  <svg className="animate-spin h-6 w-6 text-primary" fill="none" viewBox="0 0 24 24">
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                </div>
              </td>
            </tr>
          ) : data.length === 0 ? (
            <tr>
              <td
                colSpan={columns.length}
                className="px-6 py-12 text-center text-gray-500 dark:text-gray-400"
              >
                {emptyMessage}
              </td>
            </tr>
          ) : (
            data.map((record, index) => (
              <tr
                key={index}
                className="hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
              >
                {columns.map((column) => (
                  <td
                    key={column.key}
                    className={cn(
                      'px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100',
                      column.className
                    )}
                  >
                    {column.render
                      ? column.render((record as any)[column.key], record, index)
                      : (record as any)[column.key]}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
};

export const TableHead: React.FC<React.HTMLAttributes<HTMLTableSectionCellElement>> = ({
  className,
  children,
  ...props
}) => (
  <th
    className={cn(
      'px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider dark:text-gray-400',
      className
    )}
    {...props}
  >
    {children}
  </th>
);

export const TableData: React.FC<React.HTMLAttributes<HTMLTableDataCellElement>> = ({
  className,
  children,
  ...props
}) => (
  <td
    className={cn(
      'px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100',
      className
    )}
    {...props}
  >
    {children}
  </td>
);

export const TableHeader: React.FC<React.HTMLAttributes<HTMLTableSectionElement>> = ({
  className,
  children,
  ...props
}) => (
  <thead className={cn('bg-gray-50 dark:bg-gray-800', className)} {...props}>
    {children}
  </thead>
);

export const TableBody: React.FC<React.HTMLAttributes<HTMLTableSectionElement>> = ({
  className,
  children,
  ...props
}) => (
  <tbody className={cn('bg-white divide-y divide-gray-200 dark:bg-gray-900 dark:divide-gray-700', className)} {...props}>
    {children}
  </tbody>
);

export const TableRow: React.FC<React.HTMLAttributes<HTMLTableRowElement>> = ({
  className,
  children,
  ...props
}) => (
  <tr className={cn('hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors', className)} {...props}>
    {children}
  </tr>
);

export default Table;
