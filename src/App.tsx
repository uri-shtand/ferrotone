import PitchDisplay from './components/PitchDisplay';
import { usePitchCapture } from './hooks/usePitchCapture';
import './App.css';

function App() {
  const { isCapturing, error, latestFrame, start, stop } = usePitchCapture();

  return (
    <main className="container">
      <h1>FerroTone</h1>
      {error && <div className="error-message">{error}</div>}
      <PitchDisplay
        isCapturing={isCapturing}
        latestFrame={latestFrame}
        onStart={start}
        onStop={stop}
      />
    </main>
  );
}

export default App;
