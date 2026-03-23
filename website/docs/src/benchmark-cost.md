      <h1>SOM Cost Analysis</h1>

      <p>How much does SOM compression save on LLM input costs? We tested <strong>49 real-world websites</strong> and measured the token reduction from raw HTML to Plasmate SOM output.</p>

      <div class="hero-stats">
        <div class="stat-card">
          <div class="stat-value">16.6x</div>
          <div class="stat-label">Overall Compression</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">10.5x</div>
          <div class="stat-label">Median Compression</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">94%</div>
          <div class="stat-label">Token Cost Savings</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">49</div>
          <div class="stat-label">URLs Tested</div>
        </div>
      </div>

      <p>Range: <strong>0.7x</strong> (example.com, tiny page) to <strong>116.9x</strong> (cloud.google.com)</p>

      <hr>

      <h2>Cost per 1,000 Page Loads (49 URLs)</h2>

      <p>What it costs to send the content of all 49 test URLs to an LLM, repeated 1,000 times:</p>

      <table>
        <thead>
          <tr>
            <th>Model</th>
            <th>HTML Cost</th>
            <th>SOM Cost</th>
            <th>Savings</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>GPT-4 ($10/M)</td>
            <td>$50,397</td>
            <td>$3,042</td>
            <td class="savings">$47,355 (94%)</td>
          </tr>
          <tr>
            <td>GPT-4o ($2.50/M)</td>
            <td>$12,599</td>
            <td>$761</td>
            <td class="savings">$11,839 (94%)</td>
          </tr>
          <tr>
            <td>Claude Sonnet ($3/M)</td>
            <td>$15,119</td>
            <td>$913</td>
            <td class="savings">$14,207 (94%)</td>
          </tr>
        </tbody>
      </table>

      <h2>At Scale: 1M Pages/Month</h2>

      <p>Monthly cost for processing one million pages at the average page size from this benchmark:</p>

      <table>
        <thead>
          <tr>
            <th>Model</th>
            <th>HTML/month</th>
            <th>SOM/month</th>
            <th>Monthly Savings</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>GPT-4</td>
            <td>$1,028</td>
            <td>$62</td>
            <td class="savings">$966</td>
          </tr>
          <tr>
            <td>GPT-4o</td>
            <td>$257</td>
            <td>$16</td>
            <td class="savings">$241</td>
          </tr>
          <tr>
            <td>Sonnet</td>
            <td>$308</td>
            <td>$19</td>
            <td class="savings">$290</td>
          </tr>
        </tbody>
      </table>

      <hr>

      <h2>Top 15 Compression Results</h2>

      <div style="margin: 18px 0 28px;">
        <div class="compression-row">
          <span class="site-name">cloud.google.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 100%;"></div></div>
          <span class="ratio">116.9x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">linear.app</span>
          <div class="bar-wrap"><div class="bar" style="width: 90.1%;"></div></div>
          <span class="ratio">105.3x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">reddit.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 88.8%;"></div></div>
          <span class="ratio">103.8x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">figma.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 54.4%;"></div></div>
          <span class="ratio">63.6x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">tailwindcss.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 45.9%;"></div></div>
          <span class="ratio">53.7x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">nodejs.org</span>
          <div class="bar-wrap"><div class="bar" style="width: 36.5%;"></div></div>
          <span class="ratio">42.7x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">vercel.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 30.5%;"></div></div>
          <span class="ratio">35.7x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">wired.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 29.9%;"></div></div>
          <span class="ratio">35.0x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">aws.amazon.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 26.9%;"></div></div>
          <span class="ratio">31.4x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">mongodb.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 25.0%;"></div></div>
          <span class="ratio">29.2x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">typescriptlang.org</span>
          <div class="bar-wrap"><div class="bar" style="width: 23.7%;"></div></div>
          <span class="ratio">27.7x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">theguardian.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 17.4%;"></div></div>
          <span class="ratio">20.3x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">azure.microsoft.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 13.4%;"></div></div>
          <span class="ratio">15.7x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">nytimes.com</span>
          <div class="bar-wrap"><div class="bar" style="width: 13.1%;"></div></div>
          <span class="ratio">15.3x</span>
        </div>
        <div class="compression-row">
          <span class="site-name">stripe.com/docs</span>
          <div class="bar-wrap"><div class="bar" style="width: 12.5%;"></div></div>
          <span class="ratio">14.6x</span>
        </div>
      </div>

      <h3>Token Counts</h3>

      <table>
        <thead>
          <tr>
            <th>Site</th>
            <th>HTML Tokens</th>
            <th>SOM Tokens</th>
            <th>Ratio</th>
          </tr>
        </thead>
        <tbody>
          <tr><td>cloud.google.com</td><td>464,616</td><td>3,973</td><td><strong>116.9x</strong></td></tr>
          <tr><td>linear.app</td><td>562,922</td><td>5,347</td><td><strong>105.3x</strong></td></tr>
          <tr><td>reddit.com</td><td>121,027</td><td>1,166</td><td><strong>103.8x</strong></td></tr>
          <tr><td>figma.com</td><td>369,124</td><td>5,806</td><td><strong>63.6x</strong></td></tr>
          <tr><td>tailwindcss.com</td><td>233,071</td><td>4,338</td><td><strong>53.7x</strong></td></tr>
          <tr><td>nodejs.org</td><td>115,771</td><td>2,709</td><td><strong>42.7x</strong></td></tr>
          <tr><td>vercel.com</td><td>198,761</td><td>5,565</td><td><strong>35.7x</strong></td></tr>
          <tr><td>wired.com</td><td>350,411</td><td>10,006</td><td><strong>35.0x</strong></td></tr>
          <tr><td>aws.amazon.com</td><td>81,318</td><td>2,588</td><td><strong>31.4x</strong></td></tr>
          <tr><td>mongodb.com</td><td>232,141</td><td>7,944</td><td><strong>29.2x</strong></td></tr>
          <tr><td>typescriptlang.org</td><td>64,899</td><td>2,345</td><td><strong>27.7x</strong></td></tr>
          <tr><td>theguardian.com</td><td>319,879</td><td>15,753</td><td><strong>20.3x</strong></td></tr>
          <tr><td>azure.microsoft.com</td><td>134,654</td><td>8,555</td><td><strong>15.7x</strong></td></tr>
          <tr><td>nytimes.com</td><td>48,042</td><td>3,133</td><td><strong>15.3x</strong></td></tr>
          <tr><td>stripe.com/docs</td><td>48,378</td><td>3,305</td><td><strong>14.6x</strong></td></tr>
        </tbody>
      </table>

      <hr>

      <h2>Methodology</h2>

      <ul>
        <li>Each URL fetched once via <code>plasmate fetch &lt;url&gt;</code></li>
        <li>HTML token count: raw HTML bytes / 4 (standard byte-to-token estimate)</li>
        <li>SOM token count: SOM JSON output bytes / 4</li>
        <li>Compression ratio: HTML tokens / SOM tokens</li>
        <li>Cost calculated at published model input pricing</li>
        <li>No caching, no warm-up, single fetch per URL</li>
        <li>2 URLs failed (w3.org, dev.to) and were excluded</li>
      </ul>

      <h2>Reproduce</h2>

      <pre><code>cargo install plasmate
git clone https://github.com/plasmate-labs/plasmate
cd plasmate
./benchmarks/run-cost-analysis.sh</code></pre>
