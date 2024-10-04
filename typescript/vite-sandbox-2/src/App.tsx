import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { format } from 'date-fns';
import _ from 'lodash';
import { v4 as uuidv4 } from 'uuid';

const App: React.FC = () => {
  const [data, setData] = useState(null);

  useEffect(() => {
    // API リクエストの実行
    axios.get('https://jsonplaceholder.typicode.com/posts/1')
      .then(response => {
        setData(response.data);
      })
      .catch(error => {
        console.error('Error fetching data:', error);
      });

    console.log(`Current date is: ${format(new Date(), 'yyyy-MM-dd')}`);
    console.log(`Generated UUID: ${uuidv4()}`);
    console.log(_.join(['Hello', 'Vite', 'React', 'Plugin'], ' '));
  }, []);

  const loadLazyModule = () => {
    import('./LazyModule').then(({ lazyFunction }) => {
      lazyFunction();
    });
  };

  return (
    <div>
      <h1>React + Vite Plugin Test</h1>
      <button onClick={loadLazyModule}>Load Lazy Module</button>
      {data && (
        <div>
          <h2>Fetched Data:</h2>
          <pre>{JSON.stringify(data, null, 2)}</pre>
        </div>
      )}
    </div>
  );
};

export default App;
