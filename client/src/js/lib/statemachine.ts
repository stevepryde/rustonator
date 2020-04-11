export type StateTransitionFunc = (data: any) => void;

export class StateMachine {
  curState: number = -1;
  transitions: { [x: string]: StateTransitionFunc } = {};
  constructor() {}

  setInitialState(newState: number) {
    if (this.curState !== -1) {
      console.error("Cannot set initial state. curState already set!");
    }

    this.clear(newState);
    // There is no initial transition function.
  }

  clear(initialState: number) {
    this.transitions = {};
    this.curState = initialState;
  }

  getStateKey(oldState: number, newState: number): string {
    return `${oldState}:${newState}`;
  }

  setState(newState: number, data: any = null) {
    let stateKey = this.getStateKey(this.curState, newState);
    if (!this.transitions.hasOwnProperty(stateKey)) {
      console.error(`Invalid state transition from ${this.curState} to ${newState}`);
      return;
    }
    console.log("State transitioning to " + newState);

    window.setTimeout(() => {
      this.curState = newState;
      this.transitions[stateKey](data);
    }, 1);
  }

  addTransition(oldState: number, newState: number, func: StateTransitionFunc) {
    let stateKey = this.getStateKey(oldState, newState);
    if (this.transitions.hasOwnProperty(stateKey)) {
      console.error(`State transition already exists for ${oldState} -> ${newState}`);
    }

    this.transitions[stateKey] = func;
  }
}
