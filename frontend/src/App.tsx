// frontend/src/App.tsx
import "./App.css";
import ErrorBoundary from "./components/common/ErrorBoundary";
import AdminConsole from "./components/App/AdminConsole";

function App() {
  return (
    <ErrorBoundary>
      <AdminConsole />
    </ErrorBoundary>
  );
}

export default App;
