import { useState } from 'react';
import './App.css';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import Table from './components/Table';
import { invoke } from '@tauri-apps/api/core';
import ResultsTable from './components/ResultsTable';
import { SubmitHandler, useForm } from 'react-hook-form';

export type ClusteredSentences = Record<string, string[]>;

interface ClusterValues {
  minClusterSize: number;
  minSamples: number;
}

function App() {
  const [lines, setLines] = useState<string[]>([]);
  const [clusters, setClusters] = useState<ClusteredSentences | null>(null);
  const [loading, setLoading] = useState(false);
  // Processing Settings
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ClusterValues>({
    defaultValues: {
      minClusterSize: 5,
      minSamples: 3,
    },
  });

  const [, setFormValues] = useState({
    clusterSize: 5,
    minSamples: 3,
  });

  const onSubmit: SubmitHandler<ClusterValues> = async (data) => {
    try {
      setLoading(true);
      const response = await invoke<ClusteredSentences>('process_strings', {
        lines,
        minClusterSize: data.minClusterSize,
        minSamples: data.minSamples,
      });
      const clean_response = renameKeys(response);
      console.log(clean_response);
      setClusters(clean_response); // Store the response in state
      setLoading(false);
      setFormValues({
        clusterSize: 5,
        minSamples: 3,
      });
    } catch (error) {
      console.error('Failed to process values:', error);
    }
  };

  const handleReset = () => {
    setLines([]);
    setClusters(null);
    reset(); // Resets the form to default values
    setFormValues({
      clusterSize: 5,
      minSamples: 3,
    });
  };

  const handleExtractClipboard = async () => {
    try {
      setLoading(true);
      const clipboard = await readText();
      console.log(clipboard);
      let clipboardLines = clipboard.split(/\r?\n/);
      setLines(clipboardLines);
      setLoading(false);
    } catch (error) {
      console.error('Failed to read clipboard content:', error);
    }
  };

  const handleFileSelection = async () => {
    try {
      setLoading(true);
      const response = await invoke<string[]>('from_csv');
      setLines(response);
      setLoading(false);
    } catch (error) {
      console.error('Failed to process values:', error);
    }
  };

  const handleExport = async () => {
    try {
      setLoading(true);
      console.log('Invoke started at:', new Date().toISOString());
      await invoke('export_findings', {
        data: clusters,
      });
      console.log('FINDINGS EXPORTED at:', new Date().toISOString());
      setLoading(false);
    } catch (error) {
      console.error('Failed to process values:', error);
    }
  };

  return (
    <main>
      <div className="flex h-screen w-screen items-center justify-center bg-base-300">
        {/* Primary Content Area - Starting Page */}
        {lines.length < 1 && !loading && (
          <div className="flex flex-col space-y-8 p-8">
            <h1 className="text-center text-3xl font-bold">Semantic Parser</h1>
            <div className="space-y-4">
              <h2 className="text-lg">
                This tool lets you convert a table of open text from excel into
                semantically-clustered groups
              </h2>
              <h2 className="text-lg">
                If your data is simple (single lines) simply copy the list of
                values from your table in excel onto your clipboard (Ctrl + C)
                and then press the{' '}
                <kbd className="kbd kbd-sm">Extract Clipboard</kbd> button
                below. Exclude titles or any other extraneous data when copying
                over
              </h2>
              <h2 className="text-lg">
                If your data is complex or multi-line, create a CSV file, add a
                header value in cell A1, then put all values to analyze also in
                Column A and upload it using the{' '}
                <kbd className="kbd kbd-sm">Upload CSV File</kbd> button
              </h2>
              <h2 className="text-lg">
                {`Exclude titles or any other extraneous data when copying over. This tool works best on datasets where n > 25 and where there are semantically divergent topics`}
              </h2>
            </div>
            <button
              className="btn btn-primary"
              onClick={handleExtractClipboard}
            >
              Extract Clipboard
            </button>
            <button className="btn btn-secondary" onClick={handleFileSelection}>
              Upload CSV File
            </button>
          </div>
        )}
        {/* Primary Content Area - Clipboard Value Viewer */}
        {lines.length >= 1 && clusters === null && !loading && (
          <div className="flex h-screen flex-col space-y-8 p-8">
            <h1 className="flex justify-center text-center text-3xl font-bold">
              Values Extracted From Source
            </h1>
            <div className="flex grow flex-col overflow-scroll border-2 border-base-100">
              <Table lines={lines} />
            </div>
            {lines.length < 5 && (
              <div className="animate-pulse text-center font-bold text-error">
                You need to submit more than 5 values to use the tool
              </div>
            )}
            <form
              onSubmit={handleSubmit(onSubmit)}
              className="flex flex-row items-end justify-center space-x-2"
            >
              {/* Cluster Size */}
              <div className="form-control w-1/4">
                <label className="label">
                  <span className="label-text">Min Cluster Size</span>
                </label>
                <input
                  type="number"
                  className={`input input-xs input-bordered ${errors.minClusterSize ? 'input-error' : ''}`}
                  {...register('minClusterSize', {
                    valueAsNumber: true,
                    required: 'Required. 5 is recommended',
                    min: { value: 1, message: 'Must be at least 1' },
                    max: { value: 100, message: 'Must not exceed 100' },
                  })}
                />
                {errors.minClusterSize && (
                  <span className="mt-1 text-xs text-error">
                    {errors.minClusterSize.message}
                  </span>
                )}
              </div>

              {/* Minimum Samples */}
              <div className="form-control w-1/4">
                <label className="label">
                  <span className="label-text">Min Samples</span>
                </label>
                <input
                  type="number"
                  className={`input input-xs input-bordered ${errors.minSamples ? 'input-error' : ''}`}
                  {...register('minSamples', {
                    valueAsNumber: true,
                    required: 'Required. 3 is recommended',
                    min: { value: 1, message: 'Must be at least 1' },
                    max: { value: 100, message: 'Must not exceed 100' },
                  })}
                />
                {errors.minSamples && (
                  <span className="mt-1 text-xs text-error">
                    {errors.minSamples.message}
                  </span>
                )}
              </div>
              <div className="space-x-2">
                {/* Submit Button */}
                <button type="submit" className="btn btn-primary btn-sm">
                  Submit
                </button>
                <button
                  type="button"
                  className="btn btn-error btn-sm"
                  onClick={handleReset}
                >
                  Reset
                </button>
              </div>
            </form>
          </div>
        )}
        {/* Export Page */}
        {lines.length >= 1 && clusters && !loading && (
          <div className="flex h-screen flex-col space-y-8 p-8">
            <h1 className="flex justify-center text-center text-3xl font-bold">
              Detected Semantic Clusters
            </h1>
            <div className="flex grow flex-col space-y-8 overflow-scroll">
              {Object.entries(clusters).map(([key, values]) => (
                <ResultsTable groupId={key} sentences={values} />
              ))}
            </div>

            <div className="flex justify-center space-x-2">
              <button className="btn btn-primary" onClick={handleExport}>
                Export Clusters and Word Clouds
              </button>
              <button className="btn btn-error" onClick={handleReset}>
                Reset
              </button>
            </div>
          </div>
        )}
        {loading && (
          <div>{clusters ? 'Generating output...' : 'Loading...'}</div>
        )}
      </div>
    </main>
  );
}

export default App;

const renameKeys = (obj: ClusteredSentences): ClusteredSentences => {
  const renamed: ClusteredSentences = {};
  Object.keys(obj).forEach((key) => {
    renamed[`Cluster ${parseInt(key, 10) + 1}`] = obj[key];
  });
  return renamed;
};
