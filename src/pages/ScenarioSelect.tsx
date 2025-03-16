import React, { FC } from 'react';
import { useNavigate } from 'react-router-dom';

interface Scenario {
  id: number;
  name: string;
}

const ScenarioSelect: FC = () => {
  const navigate = useNavigate();
  const scenarios: Scenario[] = [
    { id: 1, name: '日常对话' },
    { id: 2, name: '商务会话' },
    { id: 3, name: '旅游场景' },
  ];

  const handleScenarioSelect = (scenarioId: number) => {
    navigate(`/chat/${scenarioId}`);
  };

  return (
    <div className="scenario-select">
      <h1>选择聊天场景</h1>
      <div className="scenario-list">
        {scenarios.map(scenario => (
          <button
            key={scenario.id}
            className="scenario-button"
            onClick={() => handleScenarioSelect(scenario.id)}
          >
            {scenario.name}
          </button>
        ))}
      </div>
    </div>
  );
};

export default ScenarioSelect; 