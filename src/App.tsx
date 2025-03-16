import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import ScenarioSelect from './pages/ScenarioSelect';
import ChatWindow from './pages/ChatWindow';
import './App.css';

const App = () => {
  return (
    <Router>
      <div className="app">
        <Routes>
          <Route path="/" element={<Navigate to="/scenarios" replace />} />
          <Route path="/scenarios" element={<ScenarioSelect />} />
          <Route path="/chat/:scenarioId" element={<ChatWindow />} />
        </Routes>
      </div>
    </Router>
  );
};

export default App;
