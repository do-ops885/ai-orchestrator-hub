const http = require('http');
const https = require('https');
const { performance } = require('perf_hooks');
const fs = require('fs');
const path = require('path');

class LoadTester {
    constructor(url, options = {}) {
        this.url = url;
        this.options = {
            concurrency: options.concurrency || 10,
            duration: options.duration || 30, // seconds
            timeout: options.timeout || 5000, // ms
            ...options
        };
        this.stats = {
            totalRequests: 0,
            successfulRequests: 0,
            failedRequests: 0,
            responseTimes: [],
            errors: [],
            startTime: null,
            endTime: null
        };
    }

    async run() {
        console.log(`🚀 Starting load test: ${this.url}`);
        console.log(`📊 Configuration: ${this.options.concurrency} concurrent users for ${this.options.duration}s`);
        console.log('================================================');

        this.stats.startTime = performance.now();
        const endTime = this.stats.startTime + (this.options.duration * 1000);

        // Create concurrent requests
        const promises = [];
        for (let i = 0; i < this.options.concurrency; i++) {
            promises.push(this.runWorker(endTime, i));
        }

        await Promise.allSettled(promises);
        this.stats.endTime = performance.now();

        this.printResults();
    }

    async runWorker(endTime, workerId) {
        while (performance.now() < endTime) {
            try {
                const startTime = performance.now();
                await this.makeRequest();
                const responseTime = performance.now() - startTime;

                this.stats.totalRequests++;
                this.stats.successfulRequests++;
                this.stats.responseTimes.push(responseTime);

            } catch (error) {
                this.stats.totalRequests++;
                this.stats.failedRequests++;
                this.stats.errors.push(error.message);
            }

            // Small delay between requests
            await new Promise(resolve => setTimeout(resolve, 10));
        }
    }

    makeRequest() {
        return new Promise((resolve, reject) => {
            const protocol = this.url.startsWith('https') ? https : http;
            const request = protocol.get(this.url, {
                timeout: this.options.timeout,
                headers: {
                    'User-Agent': 'AI-Orchestrator-Load-Test/1.0'
                }
            });

            request.on('response', (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => {
                    if (res.statusCode >= 200 && res.statusCode < 300) {
                        resolve(data);
                    } else {
                        reject(new Error(`HTTP ${res.statusCode}`));
                    }
                });
            });

            request.on('error', (err) => {
                reject(err);
            });

            request.on('timeout', () => {
                request.destroy();
                reject(new Error('Request timeout'));
            });
        });
    }

    printResults() {
        const duration = (this.stats.endTime - this.stats.startTime) / 1000;
        const requestsPerSecond = this.stats.totalRequests / duration;
        const successRate = (this.stats.successfulRequests / this.stats.totalRequests) * 100;

        // Calculate percentiles
        const sortedTimes = this.stats.responseTimes.sort((a, b) => a - b);
        const p50 = this.percentile(sortedTimes, 50);
        const p95 = this.percentile(sortedTimes, 95);
        const p99 = this.percentile(sortedTimes, 99);

        let output = '\n📈 Load Test Results:\n';
        output += '===================\n';
        output += `Duration: ${duration.toFixed(2)}s\n`;
        output += `Total Requests: ${this.stats.totalRequests}\n`;
        output += `Successful Requests: ${this.stats.successfulRequests}\n`;
        output += `Failed Requests: ${this.stats.failedRequests}\n`;
        output += `Requests/sec: ${requestsPerSecond.toFixed(2)}\n`;
        output += `Success Rate: ${successRate.toFixed(2)}%\n`;
        output += `Average Response Time: ${this.average(this.stats.responseTimes).toFixed(2)}ms\n`;
        output += `P50 Response Time: ${p50.toFixed(2)}ms\n`;
        output += `P95 Response Time: ${p95.toFixed(2)}ms\n`;
        output += `P99 Response Time: ${p99.toFixed(2)}ms\n`;

        if (this.stats.errors.length > 0) {
            output += '\n❌ Top Errors:\n';
            const errorCounts = {};
            this.stats.errors.forEach(error => {
                errorCounts[error] = (errorCounts[error] || 0) + 1;
            });

            Object.entries(errorCounts)
                .sort(([,a], [,b]) => b - a)
                .slice(0, 5)
                .forEach(([error, count]) => {
                    output += `  ${error}: ${count} times\n`;
                });
        }

        output += '\n✅ Load test completed!\n';

        // Write to console
        console.log(output);

        // Write to file
        try {
            const filePath = path.join(__dirname, '..', 'benchmarks', 'load_test_results.txt');
            fs.writeFileSync(filePath, output);
            console.log(`📄 Results also saved to benchmarks/load_test_results.txt`);
        } catch (error) {
            console.error('Failed to write load test results to file:', error.message);
        }
    }

    percentile(sortedArray, percentile) {
        if (sortedArray.length === 0) return 0;
        const index = (percentile / 100) * (sortedArray.length - 1);
        const lower = Math.floor(index);
        const upper = Math.ceil(index);
        const weight = index % 1;

        if (upper >= sortedArray.length) return sortedArray[sortedArray.length - 1];
        return sortedArray[lower] * (1 - weight) + sortedArray[upper] * weight;
    }

    average(array) {
        if (array.length === 0) return 0;
        return array.reduce((a, b) => a + b, 0) / array.length;
    }
}

// Run the load test
async function main() {
    const url = process.argv[2] || 'http://localhost:3000';
    const concurrency = parseInt(process.argv[3]) || 10;
    const duration = parseInt(process.argv[4]) || 30;

    const tester = new LoadTester(url, {
        concurrency,
        duration
    });

    try {
        await tester.run();
    } catch (error) {
        console.error('Load test failed:', error);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

module.exports = LoadTester;
