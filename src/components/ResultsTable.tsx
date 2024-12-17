import React from 'react';

interface TableProps {
  groupId: string;
  sentences: string[];
}

const ResultsTable: React.FC<TableProps> = ({ groupId, sentences }) => {
  return (
    <div className="flex flex-col">
      <h1 className="mb-4 text-center text-xl font-bold underline">
        {groupId}
      </h1>
      <table className="table table-zebra border-4 border-base-100">
        {/* head */}
        <thead>
          <tr>
            <th>Cluster ID</th>
            <th>Text</th>
          </tr>
        </thead>
        <tbody>
          {sentences.map((sentence) => {
            return (
              <tr key={groupId + sentence}>
                <th>{groupId}</th>
                <td>{sentence}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default ResultsTable;
