import { Component, signal } from '@angular/core';
import { CostPanel } from './features/cost-panel';
import { EntityBrowser } from './features/entity-browser';
import { ServiceOverview } from './features/service-overview';

@Component({
  selector: 'app-root',
  imports: [ServiceOverview, EntityBrowser, CostPanel],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  readonly panelOpen = signal(true);
  readonly view = signal<'overview' | 'data'>('overview');
}
