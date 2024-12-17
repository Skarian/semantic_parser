import React from 'react';

interface TableProps {
  lines: string[];
}

const Table: React.FC<TableProps> = ({ lines }) => {
  return (
    <div className="overflow-x-auto">
      <table className="table table-zebra">
        {/* head */}
        <thead>
          <tr>
            <th></th>
            <th>Extracted text Values</th>
          </tr>
        </thead>
        <tbody>
          {lines.map((line, index) => {
            return (
              <tr key={line + index}>
                <th>{index + 1}</th>
                <td>{line}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default Table;
