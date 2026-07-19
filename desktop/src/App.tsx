import { BrowserRouter, Routes, Route } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import DashboardPage from "./pages/DashboardPage";
import SettingsPage from "./pages/settings/SettingsPage";
import { ThemeProvider } from "./lib/ThemeProvider";
import { ConnectionProvider } from "./context/ConnectionContext";
import { LLMProvider } from "./context/LLMContext";

function App() {
  return (
    <ThemeProvider>
      <ConnectionProvider>
        <LLMProvider>
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<LoginPage />} />
              <Route path="/dashboard" element={<DashboardPage />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Routes>
          </BrowserRouter>
        </LLMProvider>
      </ConnectionProvider>
    </ThemeProvider>
  );
}

export default App;
